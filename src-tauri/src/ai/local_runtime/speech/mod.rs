pub(crate) use timing::{align, diarize};
mod artifact;
mod profiles;
pub(crate) use profiles::{
    transcription_budget_ms, transcription_execution_profile, transcription_profile,
};
mod execution;
mod stream;
mod timing;
mod trim;

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
            SpeechSeparation, SpeechSynthesis, SpeechSynthesizeParams, SpeechTranscribeParams,
            SpeechTranscriptArtifact, SpeechTranscription,
        },
    },
    local_resources::{
        models::catalog::{self, ModelKind},
        preparation::{PreparedModels, PreparedVoiceModel},
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
        |speech_output, work_directory, identity, _| {
            validate_identity(
                &speech_output.request_id,
                speech_output.attempt_epoch,
                &speech_output.model_spec_id,
                identity,
            )?;
            let bytes = read_artifact(
                work_directory,
                &identity.request_id,
                &speech_output.transcript_ref.file_name,
                speech_output.transcript_ref.byte_length,
                &speech_output.transcript_ref.sha256,
                MAX_TRANSCRIPT_BYTES,
                true,
            )?;
            if speech_output.transcript_ref.media_type
                != "application/vnd.knovum.speech-transcript+json"
            {
                return Err("invalid_output: transcript artifact media type differs".into());
            }
            serde_json::from_slice(&bytes).map_err(|error| format!("invalid_output: {error}"))
        },
    )
    .await
}

pub(crate) async fn synthesize<R: Runtime>(
    app: &AppHandle<R>,
    model: &PreparedModels,
    device_profile: RuntimeDeviceProfile,
    params: SpeechSynthesizeParams,
    options: SynthesisExecution<'_>,
) -> Result<SynthesizedAudio, String> {
    if device_profile.device != "cpu" {
        return Err("unsupported_device: MossNano synthesis requires CPU".into());
    }
    let SynthesisExecution {
        cancellation,
        priority,
        on_chunk,
    } = options;
    execute(
        Execution {
            app,
            model_path: co_located_model_directory(model, catalog::MOSS_NANO_MODELS)?,
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

/// Synthesis on an audio.cpp voice model: VoxCPM2 or one of the cloning families.
pub(crate) async fn synthesize_voice<R: Runtime>(
    app: &AppHandle<R>,
    model: &PreparedVoiceModel,
    device_profile: RuntimeDeviceProfile,
    params: SpeechSynthesizeParams,
    options: SynthesisExecution<'_>,
) -> Result<SynthesizedAudio, String> {
    validate_voice_profile(model, &device_profile, &params)?;
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

fn validate_voice_profile(
    model: &PreparedVoiceModel,
    device_profile: &RuntimeDeviceProfile,
    params: &SpeechSynthesizeParams,
) -> Result<(), String> {
    let model = model.model();
    if model.voice_engine() != Some(params.profile.engine) || params.profile.sample_rate != 48_000 {
        return Err("runtime_mismatch: voice request differs from its 48 kHz model profile".into());
    }
    let model_spec_id = model.voice_spec()?;
    if params.identity.model_spec_id != model_spec_id {
        return Err("runtime_mismatch: voice request differs from its prepared model".into());
    }
    if crate::ai::contracts::execution_profile::voice(model_spec_id, &device_profile.device)
        .as_deref()
        != Some(device_profile.execution_profile_id.as_str())
    {
        return Err("runtime_mismatch: voice backend differs from its frozen profile".into());
    }
    Ok(())
}

fn transcription_model_directory<'a>(
    resources: &'a PreparedModels,
    engine: &AsrEngine,
) -> Result<&'a Path, String> {
    let model = resources.transcription_model()?;
    let selected = transcription_profile(model)?;
    if std::mem::discriminant(engine) != std::mem::discriminant(&selected.engine) {
        return Err("runtime_mismatch: ASR engine differs from the platform model".into());
    }
    let kinds = model.transcription_models();
    co_located_model_directory(resources, kinds)
}

pub(super) fn co_located_model_directory<'a>(
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
        ModelKind::WhisperPart(_) => "whisper model part",
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
    ($request:ty, $response:ty, $capability:expr) => {
        impl SpeechRequest for $request {
            type Response = $response;

            fn identity(&self) -> &InferenceRequestIdentity {
                &self.identity
            }

            fn capability(&self) -> RuntimeCapability {
                $capability
            }
        }
    };
}

speech_request!(
    SpeechTranscribeParams,
    SpeechTranscription,
    RuntimeCapability::SpeechTranscription
);
speech_request!(
    SpeechSynthesizeParams,
    SpeechSynthesis,
    RuntimeCapability::SpeechSynthesis
);
speech_request!(
    SpeechAlignParams,
    SpeechAlignment,
    RuntimeCapability::SpeechAlignment
);
speech_request!(
    SpeechDiarizeParams,
    SpeechDiarization,
    RuntimeCapability::SpeechDiarization
);
speech_request!(
    SpeechSeparateParams,
    SpeechSeparation,
    RuntimeCapability::SpeechSeparation
);
