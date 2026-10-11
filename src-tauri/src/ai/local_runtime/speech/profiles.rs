use crate::ai::contracts::speech_wire::AsrEngine;
use crate::local_resources::model_catalog::LocalModelId;

pub(crate) struct TranscriptionProfile {
    pub engine: AsrEngine,
    pub device: &'static str,
    pub compute_type: &'static str,
    pub id: &'static str,
}

pub(crate) fn transcription_budget_ms(
    model: LocalModelId,
    duration_ms: u64,
) -> Result<u64, String> {
    let profile = transcription_profile(model)?;
    // Portable CPUs can execute Large V3 far slower than the quantized dictation models.
    let (factor, minimum) = if model == LocalModelId::WhisperLargeV3 && profile.device == "cpu" {
        (120, 30 * 60 * 1_000)
    } else {
        (4, 120_000)
    };
    Ok(duration_ms
        .saturating_mul(factor)
        .clamp(minimum, 6 * 60 * 60 * 1_000))
}

pub(crate) fn transcription_profile(model: LocalModelId) -> Result<TranscriptionProfile, String> {
    let metal = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    let (compute_type, id) = match (model, metal) {
        (LocalModelId::WhisperTiny, false) => ("q8_0", "whisper-tiny-cpu-q8_0-native-v2"),
        (LocalModelId::WhisperSmall, false) => ("q8_0", "whisper-small-cpu-q8_0-native-v2"),
        (LocalModelId::WhisperLargeV3, false) => ("f16", "whisper-large-v3-cpu-f16-native-v2"),
        (LocalModelId::WhisperTiny, true) => ("q8_0", "whisper-tiny-mps-q8_0-native-v2"),
        (LocalModelId::WhisperSmall, true) => ("q8_0", "whisper-small-mps-q8_0-native-v2"),
        (LocalModelId::WhisperLargeV3, true) => ("f16", "whisper-large-v3-mps-f16-native-v2"),
        _ => return Err("Selected model cannot transcribe audio".into()),
    };
    Ok(TranscriptionProfile {
        engine: AsrEngine::WhisperCpp,
        device: if metal { "mps" } else { "cpu" },
        compute_type,
        id,
    })
}

pub(crate) async fn transcription_execution_profile<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    model: LocalModelId,
    cancellation: &tokio_util::sync::CancellationToken,
    _deadline_ms: u64,
) -> Result<TranscriptionProfile, String> {
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    transcription_profile(model)
}

#[cfg(test)]
#[path = "tests/profiles.rs"]
mod tests;
