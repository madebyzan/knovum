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

/// What a model asks of the machine, stated beside the model instead of being recognised by a
/// caller. `Any` means the model stays within the ordinary application budget on every supported
/// target. `QualifiedOnGib` records the memory the model was actually qualified against, so the
/// catalogue can tell a person with a smaller machine what they are choosing.
///
/// Neither value is permission. Downloading and selecting a model are never refused for memory —
/// a download spends disk — and a run is admitted by `ai::local_runtime::device_profiles`'
/// memory manifest against the memory free at that moment. This is advice about the machine;
/// that is the decision about the run.
///
/// Every model states one of the two. There is no default, so a larger model added later cannot
/// reach the catalogue without answering the question.
#[derive(Clone, Copy)]
pub enum LocalModelMemory {
    Any,
    QualifiedOnGib(u64),
}

impl LocalModelMemory {
    /// The qualified size, reported only when this machine has less usable memory than that.
    /// An unknown machine size produces no advisory: we do not tell someone their computer is too
    /// small when we failed to measure it.
    pub(super) fn advisory_bytes(self, usable_bytes: Option<u64>) -> Option<u64> {
        let (Self::QualifiedOnGib(gib), Some(usable_bytes)) = (self, usable_bytes) else {
            return None;
        };
        let qualified = gib * 1024 * 1024 * 1024;
        (usable_bytes < qualified).then_some(qualified)
    }
}

pub(super) struct LocalModelMetadata {
    pub name: &'static str,
    pub capability: LocalModelCapability,
    pub operations: &'static [LocalModelOperation],
    pub reference_transcript: LocalModelReferenceTranscript,
    pub memory: LocalModelMemory,
}

pub(super) fn metadata(model: LocalModelId) -> LocalModelMetadata {
    use LocalModelCapability as Capability;
    use LocalModelMemory::{Any, QualifiedOnGib};
    use LocalModelOperation::{VoiceCloning, VoiceDesign};
    use LocalModelReferenceTranscript::{OptionalConfirmed, Unused};
    const NONE: &[LocalModelOperation] = &[];
    const CLONING: &[LocalModelOperation] = &[VoiceCloning];
    const VOX: &[LocalModelOperation] = &[VoiceCloning, VoiceDesign];
    let (name, capability, operations, reference_transcript, memory) = match model {
        LocalModelId::Qwen3Embedding => (
            "Qwen3 Embedding · 0.6B",
            Capability::Embedding,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::WhisperSmall => (
            "Whisper Small",
            Capability::Transcription,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::WhisperTiny => ("Whisper Tiny", Capability::Transcription, NONE, Unused, Any),
        LocalModelId::WhisperLargeV3 => (
            "Whisper Large-v3",
            Capability::Transcription,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::MossNano => (
            "MOSS-TTS-Nano · 100M",
            Capability::SpeechSynthesis,
            CLONING,
            Unused,
            Any,
        ),
        LocalModelId::VoxCpm2 => (
            "VoxCPM2 · Full",
            Capability::SpeechSynthesis,
            VOX,
            OptionalConfirmed,
            QualifiedOnGib(32),
        ),
        LocalModelId::VoxCpm2Q8 => (
            "VoxCPM2 · Q8",
            Capability::SpeechSynthesis,
            VOX,
            OptionalConfirmed,
            // Q8 keeps the qualified full-model figure until target measurements justify a lower
            // one. A smaller file is not a smaller process.
            QualifiedOnGib(32),
        ),
        LocalModelId::Qwen3Aligner => (
            "Qwen3 Forced Aligner · 0.6B / F16",
            Capability::SpeechAlignment,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::Qwen3AlignerQ8 => (
            "Qwen3 Forced Aligner · 0.6B / Q8",
            Capability::SpeechAlignment,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::SherpaDiarization => (
            "Sherpa-ONNX · Segmentation 3.0 / INT8 + ERes2Net",
            Capability::SpeechDiarization,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::SherpaDiarizationFull => (
            "Sherpa-ONNX · Segmentation 3.0 / FP32 + ERes2Net",
            Capability::SpeechDiarization,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::Spleeter2Stems => (
            "Spleeter · 2 stems / FP16",
            Capability::SpeechSeparation,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::Spleeter2StemsFull => (
            "Spleeter · 2 stems / FP32",
            Capability::SpeechSeparation,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::U2netp => (
            "U²-Net · Compact",
            Capability::ImageSegmentation,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::U2net => (
            "U²-Net · Full",
            Capability::ImageSegmentation,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::RealEsrganGeneralX4v3 => (
            "Real-ESRGAN · General x4v3",
            Capability::ImageUpscaling,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::RealEsrganX4plus => (
            "Real-ESRGAN · x4plus",
            Capability::ImageUpscaling,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::Migan => ("MI-GAN", Capability::ImageInpainting, NONE, Unused, Any),
        LocalModelId::EfficientSamTi => (
            "EfficientSAM · Ti",
            Capability::ImageSelection,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::EfficientSamS => (
            "EfficientSAM · S",
            Capability::ImageSelection,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::DepthAnythingV2SmallUint8 => (
            "Depth Anything V2 · Small / UINT8",
            Capability::ImageDepth,
            NONE,
            Unused,
            Any,
        ),
        LocalModelId::DepthAnythingV2Small => (
            "Depth Anything V2 · Small / FP32",
            Capability::ImageDepth,
            NONE,
            Unused,
            Any,
        ),
    };
    LocalModelMetadata {
        name,
        capability,
        operations,
        reference_transcript,
        memory,
    }
}
