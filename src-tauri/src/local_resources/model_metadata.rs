use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum LocalModelCapability {
    Embedding,
    Transcription,
    SpeechSynthesis,
    SpeechAlignment,
    SpeechDiarization,
    SpeechSeparation,
    ImageSegmentation,
    ImageUpscaling,
    ImageInpainting,
    ImageSelection,
    ImageDepth,
}

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
    LocalModelCapability,
    &'static [LocalModelOperation],
    LocalModelReferenceTranscript,
) {
    use LocalModelOperation::{VoiceCloning, VoiceDesign};
    use LocalModelReferenceTranscript::{OptionalConfirmed, Unused};
    const NONE: &[LocalModelOperation] = &[];
    const CLONING: &[LocalModelOperation] = &[VoiceCloning];
    const VOX: &[LocalModelOperation] = &[VoiceCloning, VoiceDesign];
    match model {
        LocalModelId::Qwen3Embedding => (
            "Qwen3 Embedding · 0.6B",
            LocalModelCapability::Embedding,
            NONE,
            Unused,
        ),
        LocalModelId::WhisperSmall => (
            "Whisper Small",
            LocalModelCapability::Transcription,
            NONE,
            Unused,
        ),
        LocalModelId::WhisperTiny => (
            "Whisper Tiny",
            LocalModelCapability::Transcription,
            NONE,
            Unused,
        ),
        LocalModelId::WhisperLargeV3 => (
            "Whisper Large-v3",
            LocalModelCapability::Transcription,
            NONE,
            Unused,
        ),
        LocalModelId::MossNano => (
            "MOSS-TTS-Nano · 100M",
            LocalModelCapability::SpeechSynthesis,
            CLONING,
            Unused,
        ),
        LocalModelId::VoxCpm2 => (
            "VoxCPM2 · Full",
            LocalModelCapability::SpeechSynthesis,
            VOX,
            OptionalConfirmed,
        ),
        LocalModelId::VoxCpm2Q8 => (
            "VoxCPM2 · Q8",
            LocalModelCapability::SpeechSynthesis,
            VOX,
            OptionalConfirmed,
        ),
        LocalModelId::Qwen3Aligner => (
            "Qwen3 Forced Aligner · 0.6B / FP32",
            LocalModelCapability::SpeechAlignment,
            NONE,
            Unused,
        ),
        LocalModelId::Qwen3AlignerQ8 => (
            "Qwen3 Forced Aligner · 0.6B / Q8",
            LocalModelCapability::SpeechAlignment,
            NONE,
            Unused,
        ),
        LocalModelId::SherpaDiarization => (
            "Sherpa-ONNX · Segmentation 3.0 / INT8 + ERes2Net",
            LocalModelCapability::SpeechDiarization,
            NONE,
            Unused,
        ),
        LocalModelId::SherpaDiarizationFull => (
            "Sherpa-ONNX · Segmentation 3.0 / FP32 + ERes2Net",
            LocalModelCapability::SpeechDiarization,
            NONE,
            Unused,
        ),
        LocalModelId::Spleeter2Stems => (
            "Spleeter · 2 stems / FP16",
            LocalModelCapability::SpeechSeparation,
            NONE,
            Unused,
        ),
        LocalModelId::Spleeter2StemsFull => (
            "Spleeter · 2 stems / FP32",
            LocalModelCapability::SpeechSeparation,
            NONE,
            Unused,
        ),
        LocalModelId::U2netp => (
            "U²-Net · Compact",
            LocalModelCapability::ImageSegmentation,
            NONE,
            Unused,
        ),
        LocalModelId::U2net => (
            "U²-Net · Full",
            LocalModelCapability::ImageSegmentation,
            NONE,
            Unused,
        ),
        LocalModelId::RealEsrganGeneralX4v3 => (
            "Real-ESRGAN · General x4v3",
            LocalModelCapability::ImageUpscaling,
            NONE,
            Unused,
        ),
        LocalModelId::RealEsrganX4plus => (
            "Real-ESRGAN · x4plus",
            LocalModelCapability::ImageUpscaling,
            NONE,
            Unused,
        ),
        LocalModelId::Migan => (
            "MI-GAN",
            LocalModelCapability::ImageInpainting,
            NONE,
            Unused,
        ),
        LocalModelId::EfficientSamTi => (
            "EfficientSAM · Ti",
            LocalModelCapability::ImageSelection,
            NONE,
            Unused,
        ),
        LocalModelId::EfficientSamS => (
            "EfficientSAM · S",
            LocalModelCapability::ImageSelection,
            NONE,
            Unused,
        ),
        LocalModelId::DepthAnythingV2SmallUint8 => (
            "Depth Anything V2 · Small / UINT8",
            LocalModelCapability::ImageDepth,
            NONE,
            Unused,
        ),
        LocalModelId::DepthAnythingV2Small => (
            "Depth Anything V2 · Small / FP32",
            LocalModelCapability::ImageDepth,
            NONE,
            Unused,
        ),
    }
}
