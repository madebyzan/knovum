mod image;
pub(crate) use image::ImageModelCatalog;
pub use image::{ImageModelFile, ImageModelPart, ImageModelVariant};
mod segmentation;
pub use segmentation::SegmentationModel;
pub(crate) use segmentation::{CoverageField, SegmentationSpec, SquareFit};
mod diarization;
mod moss_nano;
pub use diarization::{DiarizationPart, DiarizationVariant};
mod separation;
pub use moss_nano::MossNanoPart;
pub(crate) use moss_nano::MOSS_NANO_MODELS;
pub use separation::{SeparationPart, SeparationVariant};
mod qwen3_asr;
pub use qwen3_asr::{Qwen3AsrPart, Qwen3AsrVariant};
mod voices;
pub use voices::{VoicePart, VoiceVariant};
mod alignment;
pub(crate) use alignment::{
    QWEN3_ALIGNER_Q8_MODELS, QWEN3_ALIGNER_Q8_REVISION, QWEN3_FORCED_ALIGNER_REVISION,
    QWEN3_FORCED_ALIGNER_WEIGHTS,
};
pub(crate) use voxcpm2::{
    VOXCPM2_BF16_LICENSE, VOXCPM2_BF16_README, VOXCPM2_BF16_WEIGHTS, VOXCPM2_LICENSE,
    VOXCPM2_Q8_LICENSE, VOXCPM2_Q8_README, VOXCPM2_Q8_WEIGHTS, VOXCPM2_README, VOXCPM2_WEIGHTS,
};
mod voxcpm2;
pub(crate) use embedding::{
    TEXT_EMBEDDING_CONFIG, TEXT_EMBEDDING_MODULES, TEXT_EMBEDDING_POOLING_CONFIG,
    TEXT_EMBEDDING_SENTENCE_TRANSFORMERS_CONFIG, TEXT_EMBEDDING_TOKENIZER,
    TEXT_EMBEDDING_TOKENIZER_CONFIG, TEXT_EMBEDDING_WEIGHTS,
};
mod embedding;
mod whisper_variants;
pub use whisper_variants::WhisperPart;

#[derive(
    Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum WhisperVariant {
    Tiny,
    Base,
    Small,
    Medium,
    MediumFull,
    LargeV3TurboQ8,
    LargeV3Turbo,
    LargeV3,
}
use serde::{Deserialize, Serialize};

use super::{snapshots::SnapshotKind, ModelDescriptor};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ModelKind {
    ImagePart(ImageModelPart),
    DiarizationPart(DiarizationPart),
    MossNanoPart(MossNanoPart),
    WhisperPart(WhisperPart),
    ImageSegmentation(SegmentationModel),
    SeparationPart(SeparationPart),
    Qwen3AsrPart(Qwen3AsrPart),
    VoicePart(VoicePart),
    TextEmbeddingConfig,
    TextEmbeddingModules,
    TextEmbeddingPoolingConfig,
    TextEmbeddingSentenceTransformersConfig,
    TextEmbeddingTokenizer,
    TextEmbeddingTokenizerConfig,
    TextEmbeddingWeights,
    VoxCpm2Weights,
    VoxCpm2License,
    VoxCpm2Readme,
    VoxCpm2Bf16Weights,
    VoxCpm2Bf16License,
    VoxCpm2Bf16Readme,
    VoxCpm2Q8Weights,
    VoxCpm2Q8License,
    VoxCpm2Q8Readme,
    Qwen3AlignerQ8Weights,
    Qwen3AlignerQ8License,
    Qwen3AlignerQ8Readme,
    Qwen3ForcedAlignerWeights,
    Qwen3ForcedAlignerLicense,
    Qwen3ForcedAlignerReadme,
}

pub(crate) const TRANSCRIPTION_MODELS: &[ModelKind] = WhisperVariant::Small.models();
pub(crate) const TEXT_EMBEDDING_MODELS: &[ModelKind] = &[
    ModelKind::TextEmbeddingConfig,
    ModelKind::TextEmbeddingModules,
    ModelKind::TextEmbeddingPoolingConfig,
    ModelKind::TextEmbeddingSentenceTransformersConfig,
    ModelKind::TextEmbeddingTokenizer,
    ModelKind::TextEmbeddingTokenizerConfig,
    ModelKind::TextEmbeddingWeights,
];
pub(crate) const VOXCPM2_MODELS: &[ModelKind] = &[
    ModelKind::VoxCpm2Weights,
    ModelKind::VoxCpm2License,
    ModelKind::VoxCpm2Readme,
];
pub(crate) const VOXCPM2_BF16_MODELS: &[ModelKind] = &[
    ModelKind::VoxCpm2Bf16Weights,
    ModelKind::VoxCpm2Bf16License,
    ModelKind::VoxCpm2Bf16Readme,
];
pub(crate) const VOXCPM2_Q8_MODELS: &[ModelKind] = &[
    ModelKind::VoxCpm2Q8Weights,
    ModelKind::VoxCpm2Q8License,
    ModelKind::VoxCpm2Q8Readme,
];
pub(crate) const QWEN3_FORCED_ALIGNER_MODELS: &[ModelKind] = &[
    ModelKind::Qwen3ForcedAlignerWeights,
    ModelKind::Qwen3ForcedAlignerLicense,
    ModelKind::Qwen3ForcedAlignerReadme,
];
pub(crate) const SPEECH_SEPARATION_MODELS: &[ModelKind] = SeparationVariant::HtDemucs.models();
#[cfg(knovum_supported_desktop)]
pub(crate) const SPEECH_ALIGNMENT_MODELS: &[ModelKind] = QWEN3_FORCED_ALIGNER_MODELS;
#[cfg(not(knovum_supported_desktop))]
pub(crate) const SPEECH_ALIGNMENT_MODELS: &[ModelKind] = &[];

pub(crate) const fn descriptor(kind: ModelKind) -> ModelDescriptor {
    match kind {
        ModelKind::ImagePart(part) => part.descriptor(),
        ModelKind::DiarizationPart(part) => part.descriptor(),
        ModelKind::MossNanoPart(part) => part.descriptor(),
        ModelKind::WhisperPart(part) => part.descriptor(),
        ModelKind::ImageSegmentation(model) => model.descriptor(),
        ModelKind::SeparationPart(part) => part.descriptor(),
        ModelKind::Qwen3AsrPart(part) => part.descriptor(),
        ModelKind::VoicePart(part) => part.descriptor(),
        ModelKind::TextEmbeddingConfig => TEXT_EMBEDDING_CONFIG,
        ModelKind::TextEmbeddingModules => TEXT_EMBEDDING_MODULES,
        ModelKind::TextEmbeddingPoolingConfig => TEXT_EMBEDDING_POOLING_CONFIG,
        ModelKind::TextEmbeddingSentenceTransformersConfig => {
            TEXT_EMBEDDING_SENTENCE_TRANSFORMERS_CONFIG
        }
        ModelKind::TextEmbeddingTokenizer => TEXT_EMBEDDING_TOKENIZER,
        ModelKind::TextEmbeddingTokenizerConfig => TEXT_EMBEDDING_TOKENIZER_CONFIG,
        ModelKind::TextEmbeddingWeights => TEXT_EMBEDDING_WEIGHTS,
        ModelKind::VoxCpm2Weights => VOXCPM2_WEIGHTS,
        ModelKind::VoxCpm2License => VOXCPM2_LICENSE,
        ModelKind::VoxCpm2Readme => VOXCPM2_README,
        ModelKind::VoxCpm2Bf16Weights => VOXCPM2_BF16_WEIGHTS,
        ModelKind::VoxCpm2Bf16License => VOXCPM2_BF16_LICENSE,
        ModelKind::VoxCpm2Bf16Readme => VOXCPM2_BF16_README,
        ModelKind::VoxCpm2Q8Weights => VOXCPM2_Q8_WEIGHTS,
        ModelKind::VoxCpm2Q8License => VOXCPM2_Q8_LICENSE,
        ModelKind::VoxCpm2Q8Readme => VOXCPM2_Q8_README,
        ModelKind::Qwen3AlignerQ8Weights => alignment::QWEN3_ALIGNER_Q8_WEIGHTS,
        ModelKind::Qwen3AlignerQ8License => alignment::QWEN3_ALIGNER_Q8_LICENSE,
        ModelKind::Qwen3AlignerQ8Readme => alignment::QWEN3_ALIGNER_Q8_README,
        ModelKind::Qwen3ForcedAlignerLicense => alignment::QWEN3_ALIGNER_F16_LICENSE,
        ModelKind::Qwen3ForcedAlignerReadme => alignment::QWEN3_ALIGNER_F16_README,
        ModelKind::Qwen3ForcedAlignerWeights => QWEN3_FORCED_ALIGNER_WEIGHTS,
    }
}

pub(super) fn direct_url(kind: ModelKind) -> String {
    match kind {
        ModelKind::ImagePart(part) => part.url(),
        ModelKind::WhisperPart(part) => part.url(),
        ModelKind::MossNanoPart(part) => part.url(),
        ModelKind::SeparationPart(part) => part.url(),
        ModelKind::Qwen3AsrPart(part) => part.url(),
        ModelKind::VoicePart(part) => part.url(),
        ModelKind::Qwen3ForcedAlignerWeights
        | ModelKind::Qwen3ForcedAlignerLicense
        | ModelKind::Qwen3ForcedAlignerReadme
        | ModelKind::Qwen3AlignerQ8Weights
        | ModelKind::Qwen3AlignerQ8License
        | ModelKind::Qwen3AlignerQ8Readme => alignment::direct_url(kind),
        ModelKind::DiarizationPart(part) => part.url(),
        ModelKind::VoxCpm2Weights
        | ModelKind::VoxCpm2License
        | ModelKind::VoxCpm2Readme
        | ModelKind::VoxCpm2Bf16Weights
        | ModelKind::VoxCpm2Bf16License
        | ModelKind::VoxCpm2Bf16Readme
        | ModelKind::VoxCpm2Q8Weights
        | ModelKind::VoxCpm2Q8License
        | ModelKind::VoxCpm2Q8Readme => voxcpm2::direct_url(kind),
        _ => unreachable!("The resource source registry owns direct download models"),
    }
}
