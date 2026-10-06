pub(crate) use timing::{align, diarize};
mod artifact;
mod profiles;
pub(crate) use profiles::transcription_profile;
mod execution;
mod stream;
mod timing;

use std::path::Path;

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

pub(super) use self::execution::{execute, Execution, SpeechRequest};
pub(crate) use self::stream::{SynthesisExecution, SynthesizedAudio};
use self::{artifact::read_artifact, stream::finish_synthesis};
use super::admission::Priority;
use crate::{
    ai::contracts::{
        runtime::{RuntimeCapability, RuntimeDeviceProfile},
        speech_wire::{
            AsrEngine, InferenceRequestIdentity, SpeechAlignParams, SpeechAlignment,
            SpeechDiarization, SpeechDiarizeParams, SpeechProgressEvent, SpeechSeparateParams,
            SpeechSeparation, SpeechSynthesis, SpeechSynthesisEngine, SpeechSynthesizeParams,
            SpeechTranscribeParams, SpeechTranscriptArtifact, SpeechTranscription,
        },
    },
    local_resources::{
        models::catalog::{self, ModelKind},
        preparation::{PreparedKokoroModel, PreparedModels, PreparedVoxCpm2Model},
    },
};

const MAX_TRANSCRIPT_BYTES: u64 = 64 * 1024 * 1024;

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
        None,
        |result, work_directory, identity, _| {
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
                true,
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
    options: SynthesisExecution<'_>,
) -> Result<SynthesizedAudio, String> {
    if device_profile.device != "cpu" {
        return Err("unsupported_device: Kokoro synthesis requires CPU".into());
    }
    let SynthesisExecution {
        cancellation,
        priority,
        on_chunk,
    } = options;
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
        on_chunk,
        finish_synthesis,
    )
    .await
}

pub(crate) async fn synthesize_voxcpm2<R: Runtime>(
    app: &AppHandle<R>,
    model: &PreparedVoxCpm2Model,
    device_profile: RuntimeDeviceProfile,
    params: SpeechSynthesizeParams,
    options: SynthesisExecution<'_>,
) -> Result<SynthesizedAudio, String> {
    validate_voxcpm2_profile(&device_profile, &params)?;
    let SynthesisExecution {
        cancellation,
        priority,
        on_chunk,
    } = options;
    execute(
        Execution {
            app,
            model_path: model.directory(),
            device_profile,
            cancellation,
            priority,
        },
        params,
        None,
        on_chunk,
        finish_synthesis,
    )
    .await
}

fn validate_voxcpm2_profile(
    device_profile: &RuntimeDeviceProfile,
    params: &SpeechSynthesizeParams,
) -> Result<(), String> {
    if !matches!(params.profile.engine, SpeechSynthesisEngine::VoxCpm2)
        || params.profile.sample_rate != 48_000
    {
        return Err("runtime_mismatch: VoxCPM2 requires its 48 kHz synthesis profile".into());
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let (expected_device, expected_profile) = ("mps", "voxcpm2-mps-f32-v1");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let (expected_device, expected_profile) = ("cpu", "voxcpm2-cpu-bf16-v1");
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    {
        let _ = device_profile;
        Err("unsupported_device: VoxCPM2 is not qualified for this platform".into())
    }
    #[cfg(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))]
    {
        if device_profile.device != expected_device
            || device_profile.execution_profile_id != expected_profile
        {
            return Err(format!(
                "unsupported_device: VoxCPM2 requires {expected_profile} on this platform"
            ));
        }
        Ok(())
    }
}

fn transcription_model_directory<'a>(
    resources: &'a PreparedModels,
    engine: &AsrEngine,
) -> Result<&'a Path, String> {
    let kinds = match engine {
        AsrEngine::FasterWhisper => catalog::FASTER_WHISPER_MODELS,
        AsrEngine::MlxWhisper => catalog::MLX_WHISPER_MODELS,
    };
    co_located_model_directory(resources, kinds)
}

fn co_located_model_directory<'a>(
    resources: &'a PreparedModels,
    kinds: &[ModelKind],
) -> Result<&'a Path, String> {
    let anchor = resources.path(kinds[0])?;
    let directory = anchor
        .parent()
        .ok_or_else(|| "runtime_mismatch: speech model directory is missing".to_string())?;
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
    SpeechTranscription,
    RuntimeCapability::SpeechTranscription,
    "speech.transcribe"
);
speech_request!(
    SpeechSynthesizeParams,
    SpeechSynthesis,
    RuntimeCapability::SpeechSynthesis,
    "speech.synthesize"
);
speech_request!(
    SpeechAlignParams,
    SpeechAlignment,
    RuntimeCapability::SpeechAlignment,
    "speech.align"
);
speech_request!(
    SpeechDiarizeParams,
    SpeechDiarization,
    RuntimeCapability::SpeechDiarization,
    "speech.diarize"
);
speech_request!(
    SpeechSeparateParams,
    SpeechSeparation,
    RuntimeCapability::SpeechSeparation,
    "speech.separate"
);
