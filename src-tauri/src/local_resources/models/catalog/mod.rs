pub(crate) use separation::{
    SPLEETER_ACCOMPANIMENT, SPLEETER_FULL_ACCOMPANIMENT, SPLEETER_FULL_VOCALS, SPLEETER_VOCALS,
};
mod image;
pub(crate) use image::ImageModelCatalog;
pub use image::{ImageModelFile, ImageModelPart, ImageModelVariant};
mod diarization;
mod moss_nano;
pub use diarization::{DiarizationPart, DiarizationVariant};
mod separation;
pub use moss_nano::MossNanoPart;
pub(crate) use moss_nano::MOSS_NANO_MODELS;
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
    ImageSegmentationBiRefNet,
    ImageSegmentationRmbg20,
    ImageSegmentationPortrait,
    ImageSegmentationRmbg14,
    ImageSegmentationDis,
    ImageSegmentationAnime,
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
    SpleeterAccompaniment,
    SpleeterVocals,
    SpleeterFullAccompaniment,
    SpleeterFullVocals,
}

pub(crate) const TRANSCRIPTION_MODELS: &[ModelKind] = WhisperVariant::Small.models();
pub(crate) const IMAGE_SEGMENTATION_BIREFNET_MODELS: &[ModelKind] =
    &[ModelKind::ImageSegmentationBiRefNet];
pub(crate) const IMAGE_SEGMENTATION_RMBG20_MODELS: &[ModelKind] =
    &[ModelKind::ImageSegmentationRmbg20];
pub(crate) const IMAGE_SEGMENTATION_PORTRAIT_MODELS: &[ModelKind] =
    &[ModelKind::ImageSegmentationPortrait];
pub(crate) const IMAGE_SEGMENTATION_RMBG14_MODELS: &[ModelKind] =
    &[ModelKind::ImageSegmentationRmbg14];
pub(crate) const IMAGE_SEGMENTATION_DIS_MODELS: &[ModelKind] = &[ModelKind::ImageSegmentationDis];
pub(crate) const IMAGE_SEGMENTATION_ANIME_MODELS: &[ModelKind] =
    &[ModelKind::ImageSegmentationAnime];
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

pub(crate) const IMAGE_SEGMENTATION_BIREFNET: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentationBiRefNet,
    file: "onnx/model.onnx",

    sha256: "5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333",
    download_size: 224_005_088,
};

pub(crate) const IMAGE_SEGMENTATION_RMBG20: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentationRmbg20,
    file: "onnx/model.onnx",

    sha256: "6db79b5ac18a5a7af3b32f25d8e2580d8dca444d8a77249d72700eed425f269f",
    download_size: 1_023_275_319,
};

pub(crate) const IMAGE_SEGMENTATION_PORTRAIT: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentationPortrait,
    file: "onnx/model.onnx",

    sha256: "1ba1c8ff5a7bbfadc8d8d13fb11d7be793f91f23d9d466549e37a854f6668f99",
    download_size: 972_666_916,
};

pub(crate) const IMAGE_SEGMENTATION_RMBG14: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentationRmbg14,
    file: "onnx/model.onnx",

    sha256: "8cafcf770b06757c4eaced21b1a88e57fd2b66de01b8045f35f01535ba742e0f",
    download_size: 176_153_355,
};

pub(crate) const IMAGE_SEGMENTATION_DIS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentationDis,
    file: "isnet-general-use.onnx",

    sha256: "60920e99c45464f2ba57bee2ad08c919a52bbf852739e96947fbb4358c0d964a",
    download_size: 178_648_008,
};

pub(crate) const IMAGE_SEGMENTATION_ANIME: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::ImageSegmentationAnime,
    file: "isnet-anime.onnx",

    sha256: "f15622d853e8260172812b657053460e20806f04b9e05147d49af7bed31a6e99",
    download_size: 176_069_933,
};

pub(crate) const fn descriptor(kind: ModelKind) -> ModelDescriptor {
    match kind {
        ModelKind::ImagePart(part) => part.descriptor(),
        ModelKind::DiarizationPart(part) => part.descriptor(),
        ModelKind::MossNanoPart(part) => part.descriptor(),
        ModelKind::WhisperPart(part) => part.descriptor(),
        ModelKind::ImageSegmentationBiRefNet => IMAGE_SEGMENTATION_BIREFNET,
        ModelKind::ImageSegmentationRmbg20 => IMAGE_SEGMENTATION_RMBG20,
        ModelKind::ImageSegmentationPortrait => IMAGE_SEGMENTATION_PORTRAIT,
        ModelKind::ImageSegmentationRmbg14 => IMAGE_SEGMENTATION_RMBG14,
        ModelKind::ImageSegmentationDis => IMAGE_SEGMENTATION_DIS,
        ModelKind::ImageSegmentationAnime => IMAGE_SEGMENTATION_ANIME,
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
        ModelKind::SpleeterAccompaniment => SPLEETER_ACCOMPANIMENT,
        ModelKind::SpleeterVocals => SPLEETER_VOCALS,
        ModelKind::SpleeterFullAccompaniment => SPLEETER_FULL_ACCOMPANIMENT,
        ModelKind::SpleeterFullVocals => SPLEETER_FULL_VOCALS,
    }
}

pub(super) fn direct_url(kind: ModelKind) -> String {
    match kind {
        ModelKind::ImagePart(part) => part.url(),
        ModelKind::WhisperPart(part) => part.url(),
        ModelKind::MossNanoPart(part) => part.url(),
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
