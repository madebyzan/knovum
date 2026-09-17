use serde::{Deserialize, Serialize};

use super::ModelDescriptor;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ModelKind {
    Transcription,
    TranscriptionVad,
    TextEmbeddingConfig,
    TextEmbeddingTokenizer,
    TextEmbeddingWeights,
}

pub const TRANSCRIPTION: ModelDescriptor = ModelDescriptor {
    directory: "whisper",
    file: "ggml-small-q8_0.bin",
    env_key: "KNOVUM_WHISPER_MODEL_PATH",
    repository: "ggerganov/whisper.cpp",
    revision: "5359861c739e955e79d9a303bcbc70fb988958b1",
    sha256: "49c8fb02b65e6049d5fa6c04f81f53b867b5ec9540406812c643f177317f779f",
    download_size: 264_464_607,
};

pub const TRANSCRIPTION_VAD: ModelDescriptor = ModelDescriptor {
    directory: "whisper-vad",
    file: "ggml-silero-v6.2.0.bin",
    env_key: "KNOVUM_WHISPER_VAD_MODEL_PATH",
    repository: "ggml-org/whisper-vad",
    revision: "9ffd54a1e1ee413ddf265af9913beaf518d1639b",
    sha256: "2aa269b785eeb53a82983a20501ddf7c1d9c48e33ab63a41391ac6c9f7fb6987",
    download_size: 885_098,
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
        ModelKind::Transcription => TRANSCRIPTION,
        ModelKind::TranscriptionVad => TRANSCRIPTION_VAD,
        ModelKind::TextEmbeddingConfig => TEXT_EMBEDDING_CONFIG,
        ModelKind::TextEmbeddingTokenizer => TEXT_EMBEDDING_TOKENIZER,
        ModelKind::TextEmbeddingWeights => TEXT_EMBEDDING_WEIGHTS,
    }
}
