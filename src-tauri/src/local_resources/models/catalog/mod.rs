pub use separation::{SPLEETER_ACCOMPANIMENT, SPLEETER_VOCALS};
mod separation;
pub use alignment::{
    QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE, QWEN3_FORCED_ALIGNER_CONFIG,
    QWEN3_FORCED_ALIGNER_GENERATION_CONFIG, QWEN3_FORCED_ALIGNER_MERGES,
    QWEN3_FORCED_ALIGNER_PREPROCESSOR_CONFIG, QWEN3_FORCED_ALIGNER_REVISION,
    QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG, QWEN3_FORCED_ALIGNER_VOCABULARY,
    QWEN3_FORCED_ALIGNER_WEIGHTS,
};
mod alignment;
pub use voxcpm2::{
    VOXCPM2_AUDIO_VAE, VOXCPM2_CONFIG, VOXCPM2_SPECIAL_TOKENS, VOXCPM2_TOKENIZER,
    VOXCPM2_TOKENIZER_CONFIG, VOXCPM2_TOKENIZER_IMPLEMENTATION, VOXCPM2_WEIGHTS,
};
mod voxcpm2;
pub use embedding::{
    TEXT_EMBEDDING_CONFIG, TEXT_EMBEDDING_MODULES, TEXT_EMBEDDING_POOLING_CONFIG,
    TEXT_EMBEDDING_SENTENCE_TRANSFORMERS_CONFIG, TEXT_EMBEDDING_TOKENIZER,
    TEXT_EMBEDDING_TOKENIZER_CONFIG, TEXT_EMBEDDING_WEIGHTS,
};
mod embedding;
pub use transcription::{
    FASTER_WHISPER_CONFIG, FASTER_WHISPER_TOKENIZER, FASTER_WHISPER_VOCABULARY,
    FASTER_WHISPER_WEIGHTS, MLX_WHISPER_CONFIG, MLX_WHISPER_WEIGHTS,
};
mod transcription;
use serde::{Deserialize, Serialize};

use super::ModelDescriptor;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ModelKind {
    FasterWhisperConfig,
    FasterWhisperWeights,
    FasterWhisperTokenizer,
    FasterWhisperVocabulary,
    MlxWhisperConfig,
    MlxWhisperWeights,
    ImageSegmentation,
    TextEmbeddingConfig,
    TextEmbeddingModules,
    TextEmbeddingPoolingConfig,
    TextEmbeddingSentenceTransformersConfig,
    TextEmbeddingTokenizer,
    TextEmbeddingTokenizerConfig,
    TextEmbeddingWeights,
    VoxCpm2AudioVae,
    VoxCpm2Config,
    VoxCpm2Weights,
    VoxCpm2SpecialTokens,
    VoxCpm2TokenizerImplementation,
    VoxCpm2Tokenizer,
    VoxCpm2TokenizerConfig,
    Qwen3ForcedAlignerChatTemplate,
    Qwen3ForcedAlignerConfig,
    Qwen3ForcedAlignerGenerationConfig,
    Qwen3ForcedAlignerMerges,
    Qwen3ForcedAlignerWeights,
    Qwen3ForcedAlignerPreprocessorConfig,
    Qwen3ForcedAlignerTokenizerConfig,
    Qwen3ForcedAlignerVocabulary,
    SpleeterAccompaniment,
    SpleeterVocals,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const TRANSCRIPTION_MODELS: &[ModelKind] = MLX_WHISPER_MODELS;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const TRANSCRIPTION_MODELS: &[ModelKind] = FASTER_WHISPER_MODELS;
#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
pub const TRANSCRIPTION_MODELS: &[ModelKind] = &[];
pub const FASTER_WHISPER_MODELS: &[ModelKind] = &[
    ModelKind::FasterWhisperConfig,
    ModelKind::FasterWhisperWeights,
    ModelKind::FasterWhisperTokenizer,
    ModelKind::FasterWhisperVocabulary,
];
pub const MLX_WHISPER_MODELS: &[ModelKind] =
    &[ModelKind::MlxWhisperConfig, ModelKind::MlxWhisperWeights];
pub const IMAGE_SEGMENTATION_MODELS: &[ModelKind] = &[ModelKind::ImageSegmentation];
pub const TEXT_EMBEDDING_MODELS: &[ModelKind] = &[
    ModelKind::TextEmbeddingConfig,
    ModelKind::TextEmbeddingModules,
    ModelKind::TextEmbeddingPoolingConfig,
    ModelKind::TextEmbeddingSentenceTransformersConfig,
    ModelKind::TextEmbeddingTokenizer,
    ModelKind::TextEmbeddingTokenizerConfig,
    ModelKind::TextEmbeddingWeights,
];
pub const VOXCPM2_MODELS: &[ModelKind] = &[
    ModelKind::VoxCpm2AudioVae,
    ModelKind::VoxCpm2Config,
    ModelKind::VoxCpm2Weights,
    ModelKind::VoxCpm2SpecialTokens,
    ModelKind::VoxCpm2TokenizerImplementation,
    ModelKind::VoxCpm2Tokenizer,
    ModelKind::VoxCpm2TokenizerConfig,
];
pub const QWEN3_FORCED_ALIGNER_MODELS: &[ModelKind] = &[
    ModelKind::Qwen3ForcedAlignerChatTemplate,
    ModelKind::Qwen3ForcedAlignerConfig,
    ModelKind::Qwen3ForcedAlignerGenerationConfig,
    ModelKind::Qwen3ForcedAlignerMerges,
    ModelKind::Qwen3ForcedAlignerWeights,
    ModelKind::Qwen3ForcedAlignerPreprocessorConfig,
    ModelKind::Qwen3ForcedAlignerTokenizerConfig,
    ModelKind::Qwen3ForcedAlignerVocabulary,
];
pub const SPEECH_SEPARATION_MODELS: &[ModelKind] =
    &[ModelKind::SpleeterAccompaniment, ModelKind::SpleeterVocals];
#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
))]
pub const SPEECH_ALIGNMENT_MODELS: &[ModelKind] = QWEN3_FORCED_ALIGNER_MODELS;
#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
pub const SPEECH_ALIGNMENT_MODELS: &[ModelKind] = &[];

pub const IMAGE_SEGMENTATION: ModelDescriptor = ModelDescriptor {
    directory: "u2netp",
    file: "u2netp.onnx",
    env_key: "KNOVUM_IMAGE_SEGMENTATION_MODEL_PATH",
    repository: "tomjackson2023/rembg",
    revision: "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
    sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
    download_size: 4_574_861,
};

pub const fn descriptor(kind: ModelKind) -> ModelDescriptor {
    match kind {
        ModelKind::FasterWhisperConfig => FASTER_WHISPER_CONFIG,
        ModelKind::FasterWhisperWeights => FASTER_WHISPER_WEIGHTS,
        ModelKind::FasterWhisperTokenizer => FASTER_WHISPER_TOKENIZER,
        ModelKind::FasterWhisperVocabulary => FASTER_WHISPER_VOCABULARY,
        ModelKind::MlxWhisperConfig => MLX_WHISPER_CONFIG,
        ModelKind::MlxWhisperWeights => MLX_WHISPER_WEIGHTS,
        ModelKind::ImageSegmentation => IMAGE_SEGMENTATION,
        ModelKind::TextEmbeddingConfig => TEXT_EMBEDDING_CONFIG,
        ModelKind::TextEmbeddingModules => TEXT_EMBEDDING_MODULES,
        ModelKind::TextEmbeddingPoolingConfig => TEXT_EMBEDDING_POOLING_CONFIG,
        ModelKind::TextEmbeddingSentenceTransformersConfig => {
            TEXT_EMBEDDING_SENTENCE_TRANSFORMERS_CONFIG
        }
        ModelKind::TextEmbeddingTokenizer => TEXT_EMBEDDING_TOKENIZER,
        ModelKind::TextEmbeddingTokenizerConfig => TEXT_EMBEDDING_TOKENIZER_CONFIG,
        ModelKind::TextEmbeddingWeights => TEXT_EMBEDDING_WEIGHTS,
        ModelKind::VoxCpm2AudioVae => VOXCPM2_AUDIO_VAE,
        ModelKind::VoxCpm2Config => VOXCPM2_CONFIG,
        ModelKind::VoxCpm2Weights => VOXCPM2_WEIGHTS,
        ModelKind::VoxCpm2SpecialTokens => VOXCPM2_SPECIAL_TOKENS,
        ModelKind::VoxCpm2TokenizerImplementation => VOXCPM2_TOKENIZER_IMPLEMENTATION,
        ModelKind::VoxCpm2Tokenizer => VOXCPM2_TOKENIZER,
        ModelKind::VoxCpm2TokenizerConfig => VOXCPM2_TOKENIZER_CONFIG,
        ModelKind::Qwen3ForcedAlignerChatTemplate => QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE,
        ModelKind::Qwen3ForcedAlignerConfig => QWEN3_FORCED_ALIGNER_CONFIG,
        ModelKind::Qwen3ForcedAlignerGenerationConfig => QWEN3_FORCED_ALIGNER_GENERATION_CONFIG,
        ModelKind::Qwen3ForcedAlignerMerges => QWEN3_FORCED_ALIGNER_MERGES,
        ModelKind::Qwen3ForcedAlignerWeights => QWEN3_FORCED_ALIGNER_WEIGHTS,
        ModelKind::Qwen3ForcedAlignerPreprocessorConfig => QWEN3_FORCED_ALIGNER_PREPROCESSOR_CONFIG,
        ModelKind::Qwen3ForcedAlignerTokenizerConfig => QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG,
        ModelKind::Qwen3ForcedAlignerVocabulary => QWEN3_FORCED_ALIGNER_VOCABULARY,
        ModelKind::SpleeterAccompaniment => SPLEETER_ACCOMPANIMENT,
        ModelKind::SpleeterVocals => SPLEETER_VOCALS,
    }
}
