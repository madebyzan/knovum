use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use super::{admission::Priority, process::Worker};
use crate::{
    ai::contracts::{
        runtime::{
            RuntimeCancelParams, RuntimeCapability, RuntimeDeviceProfile, RuntimeInitializeParams,
            RuntimeModelPaths, PROTOCOL_VERSION,
        },
        speech_wire::{
            AsrEngine, InferenceRequestIdentity, SpeechProgressEvent, SpeechProgressEventType,
            SpeechSynthesizeParams, SpeechSynthesizeResult, SpeechTranscribeParams,
            SpeechTranscribeResult, SpeechTranscriptArtifact,
        },
    },
    local_resources::{
        models::catalog::{self, ModelKind},
        preparation::{PreparedKokoroModel, PreparedModels},
        tools,
    },
};

const CANCEL_GRACE: Duration = Duration::from_secs(5);
const MAX_TRANSCRIPT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_AUDIO_BYTES: u64 = 16 * 1024 * 1024;

pub(crate) struct SynthesizedAudio {
    pub bytes: Vec<u8>,
    pub sample_rate: u32,
    pub sample_count: u64,
    pub sha256: String,
}

struct Execution<'a, R: Runtime> {
    app: &'a AppHandle<R>,
    model_path: &'a Path,
    device_profile: RuntimeDeviceProfile,
    cancellation: &'a CancellationToken,
    priority: Priority,
}

struct WorkerStart<'a, R: Runtime> {
    app: &'a AppHandle<R>,
    runtime: &'a tools::PythonRuntimePaths,
    model_path: &'a Path,
    capability: RuntimeCapability,
    model_spec_id: String,
    device_profile: RuntimeDeviceProfile,
    deadline_ms: u64,
    cancellation: &'a CancellationToken,
    exit: &'a CancellationToken,
}

trait SpeechRequest: Serialize {
    type Response: DeserializeOwned;

    fn identity(&self) -> &InferenceRequestIdentity;
    fn capability(&self) -> RuntimeCapability;
    fn method(&self) -> &'static str;
}

pub(crate) async fn transcribe<R: Runtime>(
    app: &AppHandle<R>,
    resources: &PreparedModels,
    device_profile: RuntimeDeviceProfile,
    params: SpeechTranscribeParams,
    cancellation: &CancellationToken,
    priority: Priority,
    mut on_progress: impl FnMut(SpeechProgressEvent) + Send,
) -> Result<SpeechTranscriptArtifact, String> {
    if params.profile.device != device_profile.device {
        return Err("runtime_mismatch: ASR request and execution devices differ".into());
    }
    let model_path = transcription_model_directory(resources, &params.profile.engine)?;
    execute(
        Execution {
            app,
            model_path,
            device_profile,
            cancellation,
            priority,
        },
        params,
        Some(&mut on_progress),
        |result, work_directory, identity| {
            validate_identity(
                &result.request_id,
                result.attempt_epoch,
                &result.model_spec_id,
                identity,
            )?;
            let bytes = read_artifact(
                work_directory,
                &identity.request_id,
                &result.transcript_ref.file_name,
                result.transcript_ref.byte_length,
                &result.transcript_ref.sha256,
                MAX_TRANSCRIPT_BYTES,
            )?;
            if result.transcript_ref.media_type != "application/vnd.knovum.speech-transcript+json" {
                return Err("invalid_output: transcript artifact media type differs".into());
            }
            serde_json::from_slice(&bytes).map_err(|error| format!("invalid_output: {error}"))
        },
    )
    .await
}

pub(crate) async fn synthesize<R: Runtime>(
    app: &AppHandle<R>,
    model: &PreparedKokoroModel,
    device_profile: RuntimeDeviceProfile,
    params: SpeechSynthesizeParams,
    cancellation: &CancellationToken,
    priority: Priority,
) -> Result<SynthesizedAudio, String> {
    if device_profile.device != "cpu" {
        return Err("unsupported_device: Kokoro synthesis requires CPU".into());
    }
    execute(
        Execution {
            app,
            model_path: model.directory().as_path(),
            device_profile,
            cancellation,
            priority,
        },
        params,
        None,
        |result, work_directory, identity| {
            validate_identity(
                &result.request_id,
                result.attempt_epoch,
                &result.model_spec_id,
                identity,
            )?;
            let bytes = read_artifact(
                work_directory,
                &identity.request_id,
                &result.audio_ref.file_name,
                result.audio_ref.byte_length,
                &result.audio_ref.sha256,
                MAX_AUDIO_BYTES,
            )?;
            if result.audio_ref.media_type != "audio/wav"
                || result.audio_ref.channels != 1
                || result.audio_ref.sample_format != "pcm_s16le"
            {
                return Err("invalid_output: synthesized audio metadata differs".into());
            }
            Ok(SynthesizedAudio {
                bytes,
                sample_rate: result.audio_ref.sample_rate,
                sample_count: result.sample_count,
                sha256: result.audio_ref.sha256,
            })
        },
    )
    .await
}

fn transcription_model_directory<'a>(
    resources: &'a PreparedModels,
    engine: &AsrEngine,
) -> Result<&'a Path, String> {
    let kinds = match engine {
        AsrEngine::FasterWhisper => catalog::FASTER_WHISPER_MODELS,
        AsrEngine::MlxWhisper => catalog::MLX_WHISPER_MODELS,
    };
    let anchor = resources.path(kinds[0])?;
    let directory = anchor
        .parent()
        .ok_or_else(|| "runtime_mismatch: ASR model directory is missing".to_string())?;
    for kind in kinds {
        let path = resources.path(*kind)?;
        let expected = directory.join(catalog::descriptor(*kind).file);
        if path != expected {
            return Err(format!(
                "runtime_mismatch: prepared ASR model part {:?} is not co-located",
                model_kind_name(*kind)
            ));
        }
    }
    Ok(directory)
}

fn model_kind_name(kind: ModelKind) -> &'static str {
    match kind {
        ModelKind::FasterWhisperConfig => "faster-whisper config",
        ModelKind::FasterWhisperWeights => "faster-whisper weights",
        ModelKind::FasterWhisperTokenizer => "faster-whisper tokenizer",
        ModelKind::FasterWhisperVocabulary => "faster-whisper vocabulary",
        ModelKind::MlxWhisperConfig => "mlx-whisper config",
        ModelKind::MlxWhisperWeights => "mlx-whisper weights",
        _ => "non-ASR model",
    }
}

async fn execute<R, P, O, F>(
    execution: Execution<'_, R>,
    params: P,
    mut on_progress: Option<&mut (dyn FnMut(SpeechProgressEvent) + Send)>,
    finish: F,
) -> Result<O, String>
where
    R: Runtime,
    P: SpeechRequest,
    F: FnOnce(P::Response, &Path, &InferenceRequestIdentity) -> Result<O, String>,
{
    let Execution {
        app,
        model_path,
        device_profile,
        cancellation,
        priority,
    } = execution;
    let identity = params.identity();
    let deadline = super::deadline_duration(identity.deadline_ms)?;
    let exit = super::exit_token();
    let _permit = super::admission::acquire(priority, cancellation, exit, deadline).await?;
    let slot = super::worker_slot();
    let mut guard = tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        guard = slot.lock() => guard,
    };
    let runtime = tools::resolve_python_runtime(app)?.ok_or("runtime_missing")?;
    let capability = params.capability();
    if let Some(worker) = guard.as_mut() {
        if worker.capability != capability
            || worker.runtime_digest != runtime.digest
            || worker.model_path != model_path
            || worker.capability_spec_id != identity.model_spec_id
            || worker.device_profile != device_profile
        {
            worker.terminate().await;
            *guard = None;
        }
    }
    let work_directory = if let Some(worker) = guard.as_ref() {
        worker.work_directory().to_owned()
    } else {
        start_worker(
            WorkerStart {
                app,
                runtime: &runtime,
                model_path,
                capability,
                model_spec_id: identity.model_spec_id.clone(),
                device_profile,
                deadline_ms: identity.deadline_ms,
                cancellation,
                exit,
            },
            &mut guard,
        )
        .await?
    };
    let worker = guard.as_mut().ok_or("worker_crashed")?;
    let request_id = identity.request_id.clone();
    let attempt_epoch = identity.attempt_epoch;
    let mut last_event_seq = 0;
    let send = tokio::select! {
        biased;
        _ = exit.cancelled() => Err("cancelled".into()),
        _ = cancellation.cancelled() => Err("cancelled".into()),
        result = tokio::time::timeout(
            super::deadline_duration(identity.deadline_ms)?,
            worker.protocol.send_request(&request_id, params.method(), &params),
        ) => result.map_err(|_| "deadline_exceeded".to_owned())?,
    };
    if let Err(error) = send {
        worker.terminate().await;
        *guard = None;
        return Err(error);
    }
    let response = tokio::select! {
        biased;
        _ = exit.cancelled() => {
            cancel(worker, &request_id, attempt_epoch).await;
            *guard = None;
            return Err("cancelled".into());
        }
        _ = cancellation.cancelled() => {
            cancel(worker, &request_id, attempt_epoch).await;
            *guard = None;
            return Err("cancelled".into());
        }
        result = tokio::time::timeout(
            super::deadline_duration(identity.deadline_ms)?,
            worker.protocol.read_response_with_events::<P::Response>(
                &request_id,
                |method, value| {
                    if capability != RuntimeCapability::SpeechTranscription || method != "speech.progress" {
                        return Err("invalid_output: unexpected speech notification".into());
                    }
                    let event: SpeechProgressEvent = serde_json::from_value(value)
                        .map_err(|error| format!("invalid_output: {error}"))?;
                    if event.request_id != request_id
                        || event.attempt_epoch != attempt_epoch
                        || event.seq != last_event_seq + 1
                        || event.progress > 100
                        || !matches!(event.stage.as_str(), "loading" | "transcribing" | "publishing")
                        || !matches!(
                            (&event.event_type, &event.partial),
                            (SpeechProgressEventType::Progress, None)
                                | (SpeechProgressEventType::PartialTranscript, Some(_))
                        )
                        || event.language.as_ref().is_some_and(|language| language.is_empty() || language.len() > 64)
                        || event.partial.as_ref().is_some_and(|segment| {
                            segment.text.trim().is_empty() || segment.end_ms <= segment.start_ms
                        })
                    {
                        return Err("invalid_output: speech progress identity or stage differs".into());
                    }
                    last_event_seq = event.seq;
                    if let Some(callback) = on_progress.as_deref_mut() {
                        callback(event);
                    }
                    Ok(())
                },
            ),
        ) => match result {
            Ok(value) => value,
            Err(_) => {
                cancel(worker, &request_id, attempt_epoch).await;
                *guard = None;
                return Err("deadline_exceeded".into());
            }
        }
    };
    worker.last_used = Instant::now();
    let output = match response {
        Ok(value) => finish(value, &work_directory, identity),
        Err(error) => Err(error),
    };
    if output.is_err() {
        worker.terminate().await;
        *guard = None;
        return output;
    }
    drop(guard);
    super::reclaim_when_idle(slot);
    output
}

async fn start_worker<R: Runtime>(
    start: WorkerStart<'_, R>,
    slot: &mut Option<Worker>,
) -> Result<PathBuf, String> {
    let WorkerStart {
        app,
        runtime,
        model_path,
        capability,
        model_spec_id,
        device_profile,
        deadline_ms,
        cancellation,
        exit,
    } = start;
    let session_id = uuid::Uuid::new_v4().to_string();
    let work_directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("inference")
        .join(&session_id);
    let mut model_paths = RuntimeModelPaths {
        embedding: None,
        speech_transcription: None,
        speech_alignment: None,
        speech_diarization: None,
        speech_synthesis: None,
        image_segmentation: None,
    };
    let value = Some(model_path.to_string_lossy().into_owned());
    match capability {
        RuntimeCapability::SpeechTranscription => model_paths.speech_transcription = value,
        RuntimeCapability::SpeechAlignment => model_paths.speech_alignment = value,
        RuntimeCapability::SpeechDiarization => model_paths.speech_diarization = value,
        RuntimeCapability::SpeechSynthesis => model_paths.speech_synthesis = value,
        _ => return Err("runtime_mismatch: capability is not a speech method".into()),
    }
    let params = RuntimeInitializeParams {
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: runtime.digest.clone(),
        session_id,
        capability,
        capability_spec_id: model_spec_id,
        device_profile,
        work_directory: work_directory.to_string_lossy().into_owned(),
        model_paths,
    };
    *slot = Some(
        Worker::start(
            runtime,
            &work_directory,
            model_path,
            params,
            deadline_ms,
            cancellation,
            exit,
        )
        .await?,
    );
    Ok(work_directory)
}

fn read_artifact(
    work_directory: &Path,
    request_id: &str,
    file_name: &str,
    byte_length: u64,
    expected_sha256: &str,
    maximum_bytes: u64,
) -> Result<Vec<u8>, String> {
    if byte_length == 0 || byte_length > maximum_bytes {
        return Err("resource_exhausted: worker artifact exceeds its byte limit".into());
    }
    let relative = Path::new(file_name);
    let attempt_name = format!("attempt-{request_id}");
    let mut components = relative.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(name)) if name.to_str() == Some(attempt_name.as_str()))
        || !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
        || !file_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
    {
        return Err("invalid_output: worker artifact name is invalid".into());
    }
    let attempt_directory = work_directory.join(&attempt_name);
    super::artifact::checked_directory(&attempt_directory)?;
    let path = work_directory.join(relative);
    let mut file = super::artifact::open_regular(&path)?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("invalid_output: {error}"))?;
    if !metadata.file_type().is_file() || metadata.len() != byte_length {
        return Err("invalid_output: worker artifact metadata differs".into());
    }
    let mut bytes = Vec::with_capacity(byte_length as usize);
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("invalid_output: {error}"))?;
    if bytes.len() as u64 != byte_length {
        return Err("invalid_output: worker artifact changed while reading".into());
    }
    let digest = format!("{:x}", Sha256::digest(&bytes));
    if digest != expected_sha256 {
        return Err("invalid_output: worker artifact digest differs".into());
    }
    drop(file);
    fs::remove_file(path).map_err(|error| format!("invalid_output: {error}"))?;
    fs::remove_dir(attempt_directory).map_err(|error| format!("invalid_output: {error}"))?;
    Ok(bytes)
}

fn validate_identity(
    request_id: &str,
    attempt_epoch: u64,
    model_spec_id: &str,
    expected: &InferenceRequestIdentity,
) -> Result<(), String> {
    if request_id != expected.request_id
        || attempt_epoch != expected.attempt_epoch
        || model_spec_id != expected.model_spec_id
    {
        return Err("invalid_output: speech response identity differs".into());
    }
    Ok(())
}

async fn cancel(worker: &mut Worker, request_id: &str, attempt_epoch: u64) {
    let _ = worker
        .protocol
        .notify(
            "runtime.cancel",
            &RuntimeCancelParams {
                target_request_id: request_id.into(),
                attempt_epoch,
            },
        )
        .await;
    let _ = tokio::time::timeout(
        CANCEL_GRACE,
        worker
            .protocol
            .read_response_with_events::<serde_json::Value>(request_id, |method, value| {
                if method != "speech.progress" {
                    return Err(
                        "invalid_output: unexpected worker notification during cancellation".into(),
                    );
                }
                let event: SpeechProgressEvent = serde_json::from_value(value)
                    .map_err(|error| format!("invalid_output: {error}"))?;
                if event.request_id != request_id || event.attempt_epoch != attempt_epoch {
                    return Err(
                        "invalid_output: speech progress identity differs during cancellation"
                            .into(),
                    );
                }
                Ok(())
            }),
    )
    .await;
    worker.terminate().await;
}

macro_rules! speech_request {
    ($request:ty, $response:ty, $capability:expr, $method:literal) => {
        impl SpeechRequest for $request {
            type Response = $response;

            fn identity(&self) -> &InferenceRequestIdentity {
                &self.identity
            }

            fn capability(&self) -> RuntimeCapability {
                $capability
            }

            fn method(&self) -> &'static str {
                $method
            }
        }
    };
}

speech_request!(
    SpeechTranscribeParams,
    SpeechTranscribeResult,
    RuntimeCapability::SpeechTranscription,
    "speech.transcribe"
);
speech_request!(
    SpeechSynthesizeParams,
    SpeechSynthesizeResult,
    RuntimeCapability::SpeechSynthesis,
    "speech.synthesize"
);
