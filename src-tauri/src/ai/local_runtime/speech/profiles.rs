use crate::ai::contracts::{execution_profile, speech_wire::AsrEngine};
use crate::local_resources::model_catalog::LocalModelId;

pub(crate) struct TranscriptionProfile {
    pub engine: AsrEngine,
    pub device: &'static str,
    pub compute_type: &'static str,
    pub id: String,
}

pub(crate) fn transcription_budget_ms(
    model: LocalModelId,
    duration_ms: u64,
) -> Result<u64, String> {
    let profile = transcription_profile(model)?;
    // Portable CPUs execute ASR models far slower than dedicated GPU / Metal backends.
    let (factor, minimum) = if profile.device == "cpu" {
        match model {
            LocalModelId::WhisperLargeV3 => (120, 30 * 60 * 1_000),
            LocalModelId::WhisperSmall => (16, 3 * 60 * 1_000),
            LocalModelId::WhisperTiny => (8, 2 * 60 * 1_000),
            _ => (4, 120_000),
        }
    } else {
        (4, 120_000)
    };
    Ok(duration_ms
        .saturating_mul(factor)
        .clamp(minimum, 6 * 60 * 60 * 1_000))
}

pub(crate) fn transcription_profile(model: LocalModelId) -> Result<TranscriptionProfile, String> {
    let metal = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    let device = if metal { "mps" } else { "cpu" };
    let (variant, compute_type) = match model {
        LocalModelId::WhisperTiny => ("tiny", "q8_0"),
        LocalModelId::WhisperSmall => ("small", "q8_0"),
        LocalModelId::WhisperLargeV3 => ("large-v3", "f16"),
        _ => return Err("Selected model cannot transcribe audio".into()),
    };
    Ok(TranscriptionProfile {
        engine: AsrEngine::WhisperCpp,
        device,
        compute_type,
        id: execution_profile::whisper(variant, device, compute_type),
    })
}

pub(crate) async fn transcription_execution_profile<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    model: LocalModelId,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<TranscriptionProfile, String> {
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    transcription_profile(model)
}

#[cfg(test)]
#[path = "tests/profiles.rs"]
mod tests;
