use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

pub(crate) const VOXCPM2_AUDIO_VAE: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "audiovae.pth",
    env_key: "KNOVUM_VOXCPM2_AUDIO_VAE_PATH",
    sha256: "94b5d51e107e0507d4acc976cfdadb64edd6fd06d1f751dadbf2fd1594274bf1",
    download_size: 376_951_122,
};

pub(crate) const VOXCPM2_CONFIG: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "config.json",
    env_key: "KNOVUM_VOXCPM2_CONFIG_PATH",
    sha256: "405f0dcd92f7feba6011ed4eac5c8d4f74cba9712f07fd5cfa3063bbdd95402c",
    download_size: 4_336,
};

pub(crate) const VOXCPM2_WEIGHTS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "model.safetensors",
    env_key: "KNOVUM_VOXCPM2_WEIGHTS_PATH",
    sha256: "f7f964cfa9da23653baec6e6f7750719977ad944ed9f95fe52fe3a620506891d",
    download_size: 4_580_080_592,
};

pub(crate) const VOXCPM2_SPECIAL_TOKENS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "special_tokens_map.json",
    env_key: "KNOVUM_VOXCPM2_SPECIAL_TOKENS_PATH",
    sha256: "068594063e37662c02b21acf42ebb334ef6a74fb810e68a2368f88f08351de76",
    download_size: 1_632,
};

pub(crate) const VOXCPM2_TOKENIZER_IMPLEMENTATION: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "tokenization_voxcpm2.py",
    env_key: "KNOVUM_VOXCPM2_TOKENIZER_IMPLEMENTATION_PATH",
    sha256: "84489ea32b6ee0cae22ed5480cacb6df85c46624c3119be9a2021c3649a12729",
    download_size: 2_895,
};

pub(crate) const VOXCPM2_TOKENIZER: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "tokenizer.json",
    env_key: "KNOVUM_VOXCPM2_TOKENIZER_PATH",
    sha256: "f8984687e4a92a3503d521396d454b7d68e9fdaab2a0288eb3536c7c1aa4bc20",
    download_size: 3_676_772,
};

pub(crate) const VOXCPM2_TOKENIZER_CONFIG: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "tokenizer_config.json",
    env_key: "KNOVUM_VOXCPM2_TOKENIZER_CONFIG_PATH",
    sha256: "e78a3ebb48a0b9437efd1823b6b726c823da89e49dd8bcc90c02419d9baa772b",
    download_size: 5_059,
};
