mod artifact;
mod execution;
mod stream;

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
            AsrEngine, InferenceRequestIdentity, SpeechAlignParams, SpeechAlignResult,
            SpeechDiarizeParams, SpeechDiarizeResult, SpeechProgressEvent, SpeechSeparateParams,
            SpeechSeparateResult, SpeechSynthesisEngine, SpeechSynthesizeParams,
            SpeechSynthesizeResult, SpeechTranscribeParams, SpeechTranscribeResult,
            SpeechTranscriptArtifact,
        },
    },
    local_resources::{
        models::catalog::{self, ModelKind},
        preparation::{
            PreparedDiarizationModel, PreparedKokoroModel, PreparedModels, PreparedVoxCpm2Model,
        },
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

pub(crate) async fn align<R: Runtime>(
    app: &AppHandle<R>,
    resources: &PreparedModels,
    device_profile: RuntimeDeviceProfile,
    params: SpeechAlignParams,
    cancellation: &CancellationToken,
    priority: Priority,
) -> Result<SpeechAlignResult, String> {
    validate_alignment_platform()?;
    if device_profile.device != "cpu" || params.profile.device != "cpu" {
        return Err("unsupported_device: Qwen3 forced alignment requires CPU".into());
    }
    if params.text_segments.is_empty() || params.text_segments.len() > 8 {
        return Err("invalid_params: forced alignment requires one to eight segments".into());
    }
    let requested_segments = params
        .text_segments
        .iter()
        .map(|segment| segment.segment_id.clone())
        .collect::<std::collections::HashSet<_>>();
    if requested_segments.len() != params.text_segments.len() {
        return Err("invalid_params: forced alignment segment identifiers must be unique".into());
    }
    let model_path = co_located_model_directory(resources, catalog::QWEN3_FORCED_ALIGNER_MODELS)?;
    execute(
        Execution {
            app,
            model_path,
            device_profile,
            cancellation,
            priority,
        },
        params,
        None,
        None,
        move |result: SpeechAlignResult, _, identity, _| {
            validate_identity(
                &result.request_id,
                result.attempt_epoch,
                &result.model_spec_id,
                identity,
            )?;
            if !result.coverage.is_finite() || !(0.0..=1.0).contains(&result.coverage) {
                return Err("invalid_output: forced alignment coverage is invalid".into());
            }
            for unit in &result.units {
                let alignment_state_valid =
                    match (unit.start_ms, unit.end_ms, unit.unaligned_reason.as_deref()) {
                        (Some(start), Some(end), None) => end > start,
                        (None, None, Some(_)) => true,
                        _ => false,
                    };
                if !requested_segments.contains(&unit.segment_id)
                    || unit.text.is_empty()
                    || unit
                        .score
                        .is_some_and(|score| !score.is_finite() || !(0.0..=1.0).contains(&score))
                    || !alignment_state_valid
                {
                    return Err("invalid_output: forced alignment unit is invalid".into());
                }
            }
            Ok(result)
        },
    )
    .await
}

pub(crate) async fn diarize<R: Runtime>(
    app: &AppHandle<R>,
    model: &PreparedDiarizationModel,
    device_profile: RuntimeDeviceProfile,
    params: SpeechDiarizeParams,
    cancellation: &CancellationToken,
    priority: Priority,
) -> Result<SpeechDiarizeResult, String> {
    validate_diarization_platform()?;
    if device_profile.device != "cpu"
        || device_profile.execution_profile_id != "community1-knovum-cpu-v1"
    {
        return Err("unsupported_device: Community-1 diarization requires its CPU profile".into());
    }
    let bounds = &params.speaker_count_bounds;
    if bounds.minimum.is_some_and(|value| value == 0 || value > 32)
        || bounds.maximum.is_some_and(|value| value == 0 || value > 32)
        || matches!((bounds.minimum, bounds.maximum), (Some(minimum), Some(maximum)) if minimum > maximum)
    {
        return Err("invalid_params: speaker count bounds must be within one and 32".into());
    }
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
        None,
        |result: SpeechDiarizeResult, _, identity, _| {
            validate_identity(
                &result.request_id,
                result.attempt_epoch,
                &result.model_spec_id,
                identity,
            )?;
            let mut previous_start = 0;
            for (index, turn) in result.turns.iter().enumerate() {
                if turn.speaker_id.trim().is_empty()
                    || turn.speaker_id.len() > 120
                    || turn.end_ms <= turn.start_ms
                    || (index > 0 && turn.start_ms < previous_start)
                {
                    return Err("invalid_output: diarization turn is invalid".into());
                }
                previous_start = turn.start_ms;
            }
            Ok(result)
        },
    )
    .await
}

fn validate_alignment_platform() -> Result<(), String> {
    #[cfg(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))]
    return Ok(());
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    Err("unsupported_device: Qwen3 forced alignment is not qualified for this platform".into())
}

fn validate_diarization_platform() -> Result<(), String> {
    #[cfg(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))]
    return Ok(());
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    Err("unsupported_device: Community-1 diarization is not qualified for this platform".into())
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
speech_request!(
    SpeechAlignParams,
    SpeechAlignResult,
    RuntimeCapability::SpeechAlignment,
    "speech.align"
);
speech_request!(
    SpeechDiarizeParams,
    SpeechDiarizeResult,
    RuntimeCapability::SpeechDiarization,
    "speech.diarize"
);
speech_request!(
    SpeechSeparateParams,
    SpeechSeparateResult,
    RuntimeCapability::SpeechSeparation,
    "speech.separate"
);
