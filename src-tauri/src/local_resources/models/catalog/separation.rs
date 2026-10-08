use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

pub(crate) const SPLEETER_ACCOMPANIMENT: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechSeparation,
    file: "accompaniment.fp16.onnx",

    sha256: "d14cea55793cc531a5875f5f4da08207d1c5ab9292e8e0099a104eecb014fcc0",
    download_size: 19_681_024,
};

pub(crate) const SPLEETER_VOCALS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechSeparation,
    file: "vocals.fp16.onnx",

    sha256: "24cef84aedcd1fe87c0b743ef3370ad34dc1fabf6c9014d6128a75a538c7b668",
    download_size: 19_681_017,
};

pub(crate) const SPLEETER_FULL_ACCOMPANIMENT: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechSeparationFull,
    file: "accompaniment.onnx",

    sha256: "671ace17acd3720674a2bc14de32ac6292453dec20d9eb0ba4255d4ad8e3d8c0",
    download_size: 39_318_343,
};

pub(crate) const SPLEETER_FULL_VOCALS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechSeparationFull,
    file: "vocals.onnx",

    sha256: "bdc16ab6bf6117ddd4842c19e80e40e2be188fc555295064d424616b0224ac97",
    download_size: 39_318_336,
};
