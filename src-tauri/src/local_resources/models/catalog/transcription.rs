use crate::local_resources::models::ModelDescriptor;

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
