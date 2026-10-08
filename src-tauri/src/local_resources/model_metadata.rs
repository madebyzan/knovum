use serde::Serialize;

use super::model_catalog::LocalModelId;

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum LocalModelOperation {
    VoiceCloning,
    VoiceDesign,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum LocalModelReferenceTranscript {
    Unused,
    OptionalConfirmed,
}

pub(super) fn metadata(
    model: LocalModelId,
) -> (
    &'static str,
    &'static str,
    &'static [LocalModelOperation],
    LocalModelReferenceTranscript,
) {
    use LocalModelOperation::{VoiceCloning, VoiceDesign};
    use LocalModelReferenceTranscript::{OptionalConfirmed, Unused};
    const NONE: &[LocalModelOperation] = &[];
    const CLONING: &[LocalModelOperation] = &[VoiceCloning];
    const VOX: &[LocalModelOperation] = &[VoiceCloning, VoiceDesign];
    match model {
        LocalModelId::Qwen3Embedding => ("Qwen3 Embedding · 0.6B", "embedding", NONE, Unused),
        LocalModelId::WhisperSmall => ("Whisper Small", "transcription", NONE, Unused),
        LocalModelId::WhisperTiny => ("Whisper Tiny", "transcription", NONE, Unused),
        LocalModelId::WhisperLargeV3 => ("Whisper Large-v3", "transcription", NONE, Unused),
        LocalModelId::MossNano => ("MOSS-TTS-Nano · 100M", "speechSynthesis", CLONING, Unused),
        LocalModelId::VoxCpm2 => ("VoxCPM2 · Full", "speechSynthesis", VOX, OptionalConfirmed),
        LocalModelId::VoxCpm2Q8 => ("VoxCPM2 · Q8", "speechSynthesis", VOX, OptionalConfirmed),
        LocalModelId::Qwen3Aligner => (
            "Qwen3 Forced Aligner · 0.6B / FP32",
            "speechAlignment",
            NONE,
            Unused,
        ),
        LocalModelId::Qwen3AlignerQ8 => (
            "Qwen3 Forced Aligner · 0.6B / Q8",
            "speechAlignment",
            NONE,
            Unused,
        ),
        LocalModelId::SherpaDiarization => (
            "Sherpa-ONNX · Segmentation 3.0 / INT8 + ERes2Net",
            "speechDiarization",
            NONE,
            Unused,
        ),
        LocalModelId::SherpaDiarizationFull => (
            "Sherpa-ONNX · Segmentation 3.0 / FP32 + ERes2Net",
            "speechDiarization",
            NONE,
            Unused,
        ),
        LocalModelId::Spleeter2Stems => (
            "Spleeter · 2 stems / FP16",
            "speechSeparation",
            NONE,
            Unused,
        ),
        LocalModelId::Spleeter2StemsFull => (
            "Spleeter · 2 stems / FP32",
            "speechSeparation",
            NONE,
            Unused,
        ),
        LocalModelId::U2netp => ("U²-Net · Compact", "imageSegmentation", NONE, Unused),
        LocalModelId::U2net => ("U²-Net · Full", "imageSegmentation", NONE, Unused),
    }
}
