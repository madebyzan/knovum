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

const VOXCPM2_REPOSITORY: &str = "openbmb/VoxCPM2";
const VOXCPM2_REVISION: &str = "32279effe8c19989596f05d353d1447f51d9e915";

pub const VOXCPM2_AUDIO_VAE: ModelDescriptor = ModelDescriptor {
    directory: "voxcpm2",
    file: "audiovae.pth",
    env_key: "KNOVUM_VOXCPM2_AUDIO_VAE_PATH",
    repository: VOXCPM2_REPOSITORY,
    revision: VOXCPM2_REVISION,
    sha256: "94b5d51e107e0507d4acc976cfdadb64edd6fd06d1f751dadbf2fd1594274bf1",
    download_size: 376_951_122,
};

pub const VOXCPM2_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "voxcpm2",
    file: "config.json",
    env_key: "KNOVUM_VOXCPM2_CONFIG_PATH",
    repository: VOXCPM2_REPOSITORY,
    revision: VOXCPM2_REVISION,
    sha256: "405f0dcd92f7feba6011ed4eac5c8d4f74cba9712f07fd5cfa3063bbdd95402c",
    download_size: 4_336,
};

pub const VOXCPM2_WEIGHTS: ModelDescriptor = ModelDescriptor {
    directory: "voxcpm2",
    file: "model.safetensors",
    env_key: "KNOVUM_VOXCPM2_WEIGHTS_PATH",
    repository: VOXCPM2_REPOSITORY,
    revision: VOXCPM2_REVISION,
    sha256: "f7f964cfa9da23653baec6e6f7750719977ad944ed9f95fe52fe3a620506891d",
    download_size: 4_580_080_592,
};

pub const VOXCPM2_SPECIAL_TOKENS: ModelDescriptor = ModelDescriptor {
    directory: "voxcpm2",
    file: "special_tokens_map.json",
    env_key: "KNOVUM_VOXCPM2_SPECIAL_TOKENS_PATH",
    repository: VOXCPM2_REPOSITORY,
    revision: VOXCPM2_REVISION,
    sha256: "068594063e37662c02b21acf42ebb334ef6a74fb810e68a2368f88f08351de76",
    download_size: 1_632,
};

pub const VOXCPM2_TOKENIZER_IMPLEMENTATION: ModelDescriptor = ModelDescriptor {
    directory: "voxcpm2",
    file: "tokenization_voxcpm2.py",
    env_key: "KNOVUM_VOXCPM2_TOKENIZER_IMPLEMENTATION_PATH",
    repository: VOXCPM2_REPOSITORY,
    revision: VOXCPM2_REVISION,
    sha256: "84489ea32b6ee0cae22ed5480cacb6df85c46624c3119be9a2021c3649a12729",
    download_size: 2_895,
};

pub const VOXCPM2_TOKENIZER: ModelDescriptor = ModelDescriptor {
    directory: "voxcpm2",
    file: "tokenizer.json",
    env_key: "KNOVUM_VOXCPM2_TOKENIZER_PATH",
    repository: VOXCPM2_REPOSITORY,
    revision: VOXCPM2_REVISION,
    sha256: "f8984687e4a92a3503d521396d454b7d68e9fdaab2a0288eb3536c7c1aa4bc20",
    download_size: 3_676_772,
};

pub const VOXCPM2_TOKENIZER_CONFIG: ModelDescriptor = ModelDescriptor {
    directory: "voxcpm2",
    file: "tokenizer_config.json",
    env_key: "KNOVUM_VOXCPM2_TOKENIZER_CONFIG_PATH",
    repository: VOXCPM2_REPOSITORY,
    revision: VOXCPM2_REVISION,
    sha256: "e78a3ebb48a0b9437efd1823b6b726c823da89e49dd8bcc90c02419d9baa772b",
    download_size: 5_059,
};

const QWEN3_FORCED_ALIGNER_REPOSITORY: &str = "Qwen/Qwen3-ForcedAligner-0.6B";
pub const QWEN3_FORCED_ALIGNER_REVISION: &str = "c7cbfc2048c462b0d63a45797104fc9db3ad62b7";

macro_rules! qwen3_forced_aligner_descriptor {
    ($name:ident, $file:literal, $env:literal, $sha:literal, $size:literal) => {
        pub const $name: ModelDescriptor = ModelDescriptor {
            directory: "qwen3-forced-aligner-0.6b",
            file: $file,
            env_key: $env,
            repository: QWEN3_FORCED_ALIGNER_REPOSITORY,
            revision: QWEN3_FORCED_ALIGNER_REVISION,
            sha256: $sha,
            download_size: $size,
        };
    };
}

qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE,
    "chat_template.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE_PATH",
    "75a8cfca24f00de72d796fbfed6858fc9614ef3dabd8696684cc3bc03a9c58ff",
    1_161
);

qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_CONFIG,
    "config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_CONFIG_PATH",
    "d616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09",
    5_982
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_GENERATION_CONFIG,
    "generation_config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_GENERATION_CONFIG_PATH",
    "948d089b23bca1d214e768d59c4438365665f52ec6d33678f4062206b3fbbb8c",
    115
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_MERGES,
    "merges.txt",
    "KNOVUM_QWEN3_FORCED_ALIGNER_MERGES_PATH",
    "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5",
    1_671_853
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_WEIGHTS,
    "model.safetensors",
    "KNOVUM_QWEN3_FORCED_ALIGNER_WEIGHTS_PATH",
    "47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7",
    1_835_544_544
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_PREPROCESSOR_CONFIG,
    "preprocessor_config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_PREPROCESSOR_CONFIG_PATH",
    "45e120a4eda2c20c5d7f2ea9354e63536bf35e27aa573fb7cdf78017b378770d",
    330
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG,
    "tokenizer_config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG_PATH",
    "3ab80063f8511deb9566e6ad438d17b7a6277fcffd52d92854112f19d36bd81c",
    12_666
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_VOCABULARY,
    "vocab.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_VOCABULARY_PATH",
    "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910",
    2_776_833
);

const SPLEETER_REPOSITORY: &str = "csukuangfj/sherpa-onnx-spleeter-2stems-fp16";
const SPLEETER_REVISION: &str = "93ba771920ade509f8cbd6825b1a90856c797e08";

pub const SPLEETER_ACCOMPANIMENT: ModelDescriptor = ModelDescriptor {
    directory: "spleeter-2stems-fp16",
    file: "accompaniment.fp16.onnx",
    env_key: "KNOVUM_SPLEETER_ACCOMPANIMENT_PATH",
    repository: SPLEETER_REPOSITORY,
    revision: SPLEETER_REVISION,
    sha256: "d14cea55793cc531a5875f5f4da08207d1c5ab9292e8e0099a104eecb014fcc0",
    download_size: 19_681_024,
};

pub const SPLEETER_VOCALS: ModelDescriptor = ModelDescriptor {
    directory: "spleeter-2stems-fp16",
    file: "vocals.fp16.onnx",
    env_key: "KNOVUM_SPLEETER_VOCALS_PATH",
    repository: SPLEETER_REPOSITORY,
    revision: SPLEETER_REVISION,
    sha256: "24cef84aedcd1fe87c0b743ef3370ad34dc1fabf6c9014d6128a75a538c7b668",
    download_size: 19_681_017,
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
