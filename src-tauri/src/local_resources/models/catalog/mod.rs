pub(crate) use separation::{
    SPLEETER_ACCOMPANIMENT, SPLEETER_FULL_ACCOMPANIMENT, SPLEETER_FULL_VOCALS, SPLEETER_VOCALS,
};
mod image;
pub use image::{ImageModelFile, ImageModelPart, ImageModelVariant};
mod diarization;
mod moss_nano;
pub use diarization::{DiarizationPart, DiarizationVariant};
mod separation;
pub(crate) use alignment::{
    QWEN3_ALIGNER_Q8_MODELS, QWEN3_ALIGNER_Q8_REVISION, QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE,
    QWEN3_FORCED_ALIGNER_CONFIG, QWEN3_FORCED_ALIGNER_PROCESSOR_CONFIG,
    QWEN3_FORCED_ALIGNER_REVISION, QWEN3_FORCED_ALIGNER_TOKENIZER,
    QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG, QWEN3_FORCED_ALIGNER_WEIGHTS,
};
pub use moss_nano::MossNanoPart;
pub(crate) use moss_nano::MOSS_NANO_MODELS;
mod alignment;
pub(crate) use voxcpm2::{
    VOXCPM2_LICENSE, VOXCPM2_Q8_LICENSE, VOXCPM2_Q8_README, VOXCPM2_Q8_WEIGHTS, VOXCPM2_README,
    VOXCPM2_WEIGHTS,
};
mod voxcpm2;
pub(crate) use embedding::{
    TEXT_EMBEDDING_CONFIG, TEXT_EMBEDDING_MODULES, TEXT_EMBEDDING_POOLING_CONFIG,
    TEXT_EMBEDDING_SENTENCE_TRANSFORMERS_CONFIG, TEXT_EMBEDDING_TOKENIZER,
    TEXT_EMBEDDING_TOKENIZER_CONFIG, TEXT_EMBEDDING_WEIGHTS,
};
mod embedding;
pub(crate) use transcription::{
    FASTER_WHISPER_CONFIG, FASTER_WHISPER_TOKENIZER, FASTER_WHISPER_VOCABULARY,
    FASTER_WHISPER_WEIGHTS, MLX_WHISPER_CONFIG, MLX_WHISPER_WEIGHTS,
};
mod transcription;
mod whisper_variants;
pub use whisper_variants::WhisperPart;

#[derive(
    Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum WhisperVariant {
    FasterTiny,
    FasterLargeV3,
    MlxTiny,
    MlxLargeV3,
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
    FasterWhisperConfig,
    FasterWhisperWeights,
    FasterWhisperTokenizer,
    FasterWhisperVocabulary,
    MlxWhisperConfig,
    MlxWhisperWeights,
    ImageSegmentation,
    ImageSegmentationQuality,
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
    VoxCpm2Q8Weights,
    VoxCpm2Q8License,
    VoxCpm2Q8Readme,
    Qwen3AlignerQ8Weights,
    Qwen3AlignerQ8License,
    Qwen3AlignerQ8Readme,
    Qwen3ForcedAlignerChatTemplate,
    Qwen3ForcedAlignerConfig,
    Qwen3ForcedAlignerWeights,
    Qwen3ForcedAlignerProcessorConfig,
    Qwen3ForcedAlignerTokenizer,
    Qwen3ForcedAlignerTokenizerConfig,
    SpleeterAccompaniment,
    SpleeterVocals,
    SpleeterFullAccompaniment,
    SpleeterFullVocals,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) const TRANSCRIPTION_MODELS: &[ModelKind] = MLX_WHISPER_MODELS;
#[cfg(all(
    any(target_os = "macos", target_os = "windows"),
    target_arch = "x86_64"
))]
pub(crate) const TRANSCRIPTION_MODELS: &[ModelKind] = FASTER_WHISPER_MODELS;
#[cfg(not(knovum_supported_desktop))]
pub(crate) const TRANSCRIPTION_MODELS: &[ModelKind] = &[];
pub(crate) const FASTER_WHISPER_MODELS: &[ModelKind] = &[
    ModelKind::FasterWhisperConfig,
    ModelKind::FasterWhisperWeights,
    ModelKind::FasterWhisperTokenizer,
    ModelKind::FasterWhisperVocabulary,
];
pub(crate) const MLX_WHISPER_MODELS: &[ModelKind] =
    &[ModelKind::MlxWhisperConfig, ModelKind::MlxWhisperWeights];
pub(crate) const IMAGE_SEGMENTATION_MODELS: &[ModelKind] = &[ModelKind::ImageSegmentation];
pub(crate) const IMAGE_SEGMENTATION_QUALITY_MODELS: &[ModelKind] =
    &[ModelKind::ImageSegmentationQuality];
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
pub(crate) const VOXCPM2_Q8_MODELS: &[ModelKind] = &[
    ModelKind::VoxCpm2Q8Weights,
    ModelKind::VoxCpm2Q8License,
    ModelKind::VoxCpm2Q8Readme,
];
pub(crate) const QWEN3_FORCED_ALIGNER_MODELS: &[ModelKind] = &[
    ModelKind::Qwen3ForcedAlignerChatTemplate,
    ModelKind::Qwen3ForcedAlignerConfig,
    ModelKind::Qwen3ForcedAlignerWeights,
    ModelKind::Qwen3ForcedAlignerProcessorConfig,
    ModelKind::Qwen3ForcedAlignerTokenizer,
    ModelKind::Qwen3ForcedAlignerTokenizerConfig,
];
pub(crate) const SPEECH_SEPARATION_MODELS: &[ModelKind] =
    &[ModelKind::SpleeterAccompaniment, ModelKind::SpleeterVocals];
pub(crate) const SPEECH_SEPARATION_FULL_MODELS: &[ModelKind] = &[
    ModelKind::SpleeterFullAccompaniment,
    ModelKind::SpleeterFullVocals,
];
#[cfg(knovum_supported_desktop)]
pub(crate) const SPEECH_ALIGNMENT_MODELS: &[ModelKind] = QWEN3_FORCED_ALIGNER_MODELS;
#[cfg(not(knovum_supported_desktop))]
pub(crate) const SPEECH_ALIGNMENT_MODELS: &[ModelKind] = &[];

pub(crate) const IMAGE_SEGMENTATION: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentation,
    file: "u2netp.onnx",

    sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
    download_size: 4_574_861,
};

pub(crate) const IMAGE_SEGMENTATION_QUALITY: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentationQuality,
    file: "u2net.onnx",

    sha256: "8d10d2f3bb75ae3b6d527c77944fc5e7dcd94b29809d47a739a7a728a912b491",
    download_size: 175_997_641,
};

pub(crate) const fn descriptor(kind: ModelKind) -> ModelDescriptor {
    match kind {
        ModelKind::ImagePart(part) => part.descriptor(),
        ModelKind::DiarizationPart(part) => part.descriptor(),
        ModelKind::MossNanoPart(part) => part.descriptor(),
        ModelKind::WhisperPart(part) => part.descriptor(),
        ModelKind::FasterWhisperConfig => FASTER_WHISPER_CONFIG,
        ModelKind::FasterWhisperWeights => FASTER_WHISPER_WEIGHTS,
        ModelKind::FasterWhisperTokenizer => FASTER_WHISPER_TOKENIZER,
        ModelKind::FasterWhisperVocabulary => FASTER_WHISPER_VOCABULARY,
        ModelKind::MlxWhisperConfig => MLX_WHISPER_CONFIG,
        ModelKind::MlxWhisperWeights => MLX_WHISPER_WEIGHTS,
        ModelKind::ImageSegmentation => IMAGE_SEGMENTATION,
        ModelKind::ImageSegmentationQuality => IMAGE_SEGMENTATION_QUALITY,
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
        ModelKind::VoxCpm2Q8Weights => VOXCPM2_Q8_WEIGHTS,
        ModelKind::VoxCpm2Q8License => VOXCPM2_Q8_LICENSE,
        ModelKind::VoxCpm2Q8Readme => VOXCPM2_Q8_README,
        ModelKind::Qwen3AlignerQ8Weights => alignment::QWEN3_ALIGNER_Q8_WEIGHTS,
        ModelKind::Qwen3AlignerQ8License => alignment::QWEN3_ALIGNER_Q8_LICENSE,
        ModelKind::Qwen3AlignerQ8Readme => alignment::QWEN3_ALIGNER_Q8_README,
        ModelKind::Qwen3ForcedAlignerChatTemplate => QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE,
        ModelKind::Qwen3ForcedAlignerConfig => QWEN3_FORCED_ALIGNER_CONFIG,
        ModelKind::Qwen3ForcedAlignerWeights => QWEN3_FORCED_ALIGNER_WEIGHTS,
        ModelKind::Qwen3ForcedAlignerProcessorConfig => QWEN3_FORCED_ALIGNER_PROCESSOR_CONFIG,
        ModelKind::Qwen3ForcedAlignerTokenizer => QWEN3_FORCED_ALIGNER_TOKENIZER,
        ModelKind::Qwen3ForcedAlignerTokenizerConfig => QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG,
        ModelKind::SpleeterAccompaniment => SPLEETER_ACCOMPANIMENT,
        ModelKind::SpleeterVocals => SPLEETER_VOCALS,
        ModelKind::SpleeterFullAccompaniment => SPLEETER_FULL_ACCOMPANIMENT,
        ModelKind::SpleeterFullVocals => SPLEETER_FULL_VOCALS,
    }
}

pub(super) fn direct_url(kind: ModelKind) -> String {
    match kind {
        ModelKind::ImagePart(part) => part.url(),
        ModelKind::MossNanoPart(part) => part.url(),
        ModelKind::Qwen3AlignerQ8Weights
        | ModelKind::Qwen3AlignerQ8License
        | ModelKind::Qwen3AlignerQ8Readme => alignment::direct_url(kind).into(),
        ModelKind::DiarizationPart(part) => part.url(),
        ModelKind::VoxCpm2Weights
        | ModelKind::VoxCpm2License
        | ModelKind::VoxCpm2Readme
        | ModelKind::VoxCpm2Q8Weights
        | ModelKind::VoxCpm2Q8License
        | ModelKind::VoxCpm2Q8Readme => voxcpm2::direct_url(kind).to_owned(),
        _ => unreachable!("The resource source registry owns direct download models"),
    }
}
