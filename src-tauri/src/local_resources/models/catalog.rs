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

pub const FASTER_WHISPER_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "faster-whisper-small",
    file: "config.json",
    env_key: "KNOVUM_FASTER_WHISPER_CONFIG_PATH",
    repository: "Systran/faster-whisper-small",
    revision: "536b0662742c02347bc0e980a01041f333bce120",
    sha256: "b55496ac7940a7ae47d2c01eab40edfd8701feec1229d9cce3b40014383fb828",
    download_size: 2_370,
};

pub const FASTER_WHISPER_WEIGHTS: ModelDescriptor = ModelDescriptor {
    directory: "faster-whisper-small",
    file: "model.bin",
    env_key: "KNOVUM_FASTER_WHISPER_WEIGHTS_PATH",
    repository: "Systran/faster-whisper-small",
    revision: "536b0662742c02347bc0e980a01041f333bce120",
    sha256: "3e305921506d8872816023e4c273e75d2419fb89b24da97b4fe7bce14170d671",
    download_size: 483_546_902,
};

pub const FASTER_WHISPER_TOKENIZER: ModelDescriptor = ModelDescriptor {
    directory: "faster-whisper-small",
    file: "tokenizer.json",
    env_key: "KNOVUM_FASTER_WHISPER_TOKENIZER_PATH",
    repository: "Systran/faster-whisper-small",
    revision: "536b0662742c02347bc0e980a01041f333bce120",
    sha256: "fb7b63191e9bb045082c79fd742a3106a12c99513ab30df4a0d47fa6cb6fd0ab",
    download_size: 2_203_239,
};

pub const FASTER_WHISPER_VOCABULARY: ModelDescriptor = ModelDescriptor {
    directory: "faster-whisper-small",
    file: "vocabulary.txt",
    env_key: "KNOVUM_FASTER_WHISPER_VOCABULARY_PATH",
    repository: "Systran/faster-whisper-small",
    revision: "536b0662742c02347bc0e980a01041f333bce120",
    sha256: "34ce3fe1c5041027b3f8d42912270993f986dbc4bb34cf27f951e34a1e453913",
    download_size: 459_861,
};

pub const MLX_WHISPER_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "mlx-whisper-small",
    file: "config.json",
    env_key: "KNOVUM_MLX_WHISPER_CONFIG_PATH",
    repository: "mlx-community/whisper-small-mlx",
    revision: "45f3915923c7a79a5a5b5a7d909d39aeb0e5630e",
    sha256: "e8f58e638208af66d5d5d67801259dc7a12d199e971967a9f9d33a8e3635668e",
    download_size: 266,
};

pub const MLX_WHISPER_WEIGHTS: ModelDescriptor = ModelDescriptor {
    directory: "mlx-whisper-small",
    file: "weights.npz",
    env_key: "KNOVUM_MLX_WHISPER_WEIGHTS_PATH",
    repository: "mlx-community/whisper-small-mlx",
    revision: "45f3915923c7a79a5a5b5a7d909d39aeb0e5630e",
    sha256: "55b6674c9b339702d486e2b1573839a66f8ec8f821ed2886993ef717a86b09f5",
    download_size: 481_307_592,
};

pub const IMAGE_SEGMENTATION: ModelDescriptor = ModelDescriptor {
    directory: "u2netp",
    file: "u2netp.onnx",
    env_key: "KNOVUM_IMAGE_SEGMENTATION_MODEL_PATH",
    repository: "tomjackson2023/rembg",
    revision: "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
    sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
    download_size: 4_574_861,
};

pub const TEXT_EMBEDDING_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "qwen3-embedding",
    file: "config.json",
    env_key: "KNOVUM_TEXT_EMBEDDING_CONFIG_PATH",
    repository: "Qwen/Qwen3-Embedding-0.6B",
    revision: "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
    sha256: "b5bf1f51fc45be473a54718cef92448d90a1be001bf9b9a44b8c7f10a19feaa9",
    download_size: 727,
};

pub const TEXT_EMBEDDING_TOKENIZER: ModelDescriptor = ModelDescriptor {
    directory: "qwen3-embedding",
    file: "tokenizer.json",
    env_key: "KNOVUM_TEXT_EMBEDDING_TOKENIZER_PATH",
    repository: "Qwen/Qwen3-Embedding-0.6B",
    revision: "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
    sha256: "def76fb086971c7867b829c23a26261e38d9d74e02139253b38aeb9df8b4b50a",
    download_size: 11_423_705,
};

pub const TEXT_EMBEDDING_MODULES: ModelDescriptor = ModelDescriptor {
    directory: "qwen3-embedding",
    file: "modules.json",
    env_key: "KNOVUM_TEXT_EMBEDDING_MODULES_PATH",
    repository: "Qwen/Qwen3-Embedding-0.6B",
    revision: "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
    sha256: "84e40c8e006c9b1d6c122e02cba9b02458120b5fb0c87b746c41e0207cf642cf",
    download_size: 349,
};

pub const TEXT_EMBEDDING_POOLING_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "qwen3-embedding",
    file: "1_Pooling/config.json",
    env_key: "KNOVUM_TEXT_EMBEDDING_POOLING_CONFIG_PATH",
    repository: "Qwen/Qwen3-Embedding-0.6B",
    revision: "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
    sha256: "37bf193fa101f19101bfad9c31d3eb0f786e247b7b1e5cb7f007d730eed1ddbd",
    download_size: 313,
};

pub const TEXT_EMBEDDING_SENTENCE_TRANSFORMERS_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "qwen3-embedding",
    file: "config_sentence_transformers.json",
    env_key: "KNOVUM_TEXT_EMBEDDING_SENTENCE_TRANSFORMERS_CONFIG_PATH",
    repository: "Qwen/Qwen3-Embedding-0.6B",
    revision: "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
    sha256: "10667c72ddb772627bf1780cb7f86af8e2ae0032b8c243c731172064105c6961",
    download_size: 215,
};

pub const TEXT_EMBEDDING_TOKENIZER_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "qwen3-embedding",
    file: "tokenizer_config.json",
    env_key: "KNOVUM_TEXT_EMBEDDING_TOKENIZER_CONFIG_PATH",
    repository: "Qwen/Qwen3-Embedding-0.6B",
    revision: "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
    sha256: "253153d0738ceb4c668d2eff957714dd2bea0b56de772a9fdccd96cbf517e6a0",
    download_size: 9_706,
};

pub const TEXT_EMBEDDING_WEIGHTS: ModelDescriptor = ModelDescriptor {
    directory: "qwen3-embedding",
    file: "model.safetensors",
    env_key: "KNOVUM_TEXT_EMBEDDING_WEIGHTS_PATH",
    repository: "Qwen/Qwen3-Embedding-0.6B",
    revision: "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
    sha256: "0437e45c94563b09e13cb7a64478fc406947a93cb34a7e05870fc8dcd48e23fd",
    download_size: 1_191_586_416,
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
    }
}
