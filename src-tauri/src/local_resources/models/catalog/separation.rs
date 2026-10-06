use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

pub(crate) const SPLEETER_ACCOMPANIMENT: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechSeparation,
    file: "accompaniment.fp16.onnx",
    env_key: "KNOVUM_SPLEETER_ACCOMPANIMENT_PATH",
    sha256: "d14cea55793cc531a5875f5f4da08207d1c5ab9292e8e0099a104eecb014fcc0",
    download_size: 19_681_024,
};

pub(crate) const SPLEETER_VOCALS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechSeparation,
    file: "vocals.fp16.onnx",
    env_key: "KNOVUM_SPLEETER_VOCALS_PATH",
    sha256: "24cef84aedcd1fe87c0b743ef3370ad34dc1fabf6c9014d6128a75a538c7b668",
    download_size: 19_681_017,
};
