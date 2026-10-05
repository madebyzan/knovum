use crate::local_resources::models::ModelDescriptor;

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
