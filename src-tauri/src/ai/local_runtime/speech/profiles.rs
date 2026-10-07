use crate::ai::contracts::speech_wire::AsrEngine;

pub(crate) struct TranscriptionProfile {
    pub engine: AsrEngine,
    pub device: &'static str,
    pub compute_type: &'static str,
    pub id: &'static str,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) fn transcription_profile() -> Result<TranscriptionProfile, String> {
    Ok(TranscriptionProfile {
        engine: AsrEngine::MlxWhisper,
        device: "mps",
        compute_type: "float16",
        id: "mlx-whisper-small-mps-v1",
    })
}

#[cfg(all(
    any(target_os = "macos", target_os = "windows"),
    target_arch = "x86_64"
))]
pub(crate) fn transcription_profile() -> Result<TranscriptionProfile, String> {
    Ok(TranscriptionProfile {
        engine: AsrEngine::FasterWhisper,
        device: "cpu",
        compute_type: "int8",
        id: "faster-whisper-small-cpu-int8-v1",
    })
}

#[cfg(not(knovum_supported_desktop))]
pub(crate) fn transcription_profile() -> Result<TranscriptionProfile, String> {
    Err("unsupported_device: ASR requires macOS arm64/x64 or Windows x64".into())
}

#[cfg(test)]
#[path = "tests/profiles.rs"]
mod tests;
