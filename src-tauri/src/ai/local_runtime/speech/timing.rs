use super::{co_located_model_directory, execute, validate_identity, Execution};
use crate::{
    ai::{
        contracts::{
            runtime::RuntimeDeviceProfile,
            speech_wire::{
                SpeechAlignParams, SpeechAlignment, SpeechDiarization, SpeechDiarizeParams,
            },
        },
        local_runtime::admission::Priority,
    },
    local_resources::{
        models::catalog,
        preparation::{PreparedDiarizationModel, PreparedModels},
    },
};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

pub(crate) async fn align<R: Runtime>(
    app: &AppHandle<R>,
    resources: &PreparedModels,
    device_profile: RuntimeDeviceProfile,
    params: SpeechAlignParams,
    cancellation: &CancellationToken,
    priority: Priority,
) -> Result<SpeechAlignment, String> {
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
        move |alignment: SpeechAlignment, _, identity, _| {
            validate_identity(
                &alignment.request_id,
                alignment.attempt_epoch,
                &alignment.model_spec_id,
                identity,
            )?;
            if !alignment.coverage.is_finite() || !(0.0..=1.0).contains(&alignment.coverage) {
                return Err("invalid_output: forced alignment coverage is invalid".into());
            }
            for unit in &alignment.units {
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
            Ok(alignment)
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
) -> Result<SpeechDiarization, String> {
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
        |diarization: SpeechDiarization, _, identity, _| {
            validate_identity(
                &diarization.request_id,
                diarization.attempt_epoch,
                &diarization.model_spec_id,
                identity,
            )?;
            let mut previous_start = 0;
            for (index, turn) in diarization.turns.iter().enumerate() {
                if turn.speaker_id.trim().is_empty()
                    || turn.speaker_id.len() > 120
                    || turn.end_ms <= turn.start_ms
                    || (index > 0 && turn.start_ms < previous_start)
                {
                    return Err("invalid_output: diarization turn is invalid".into());
                }
                previous_start = turn.start_ms;
            }
            Ok(diarization)
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
