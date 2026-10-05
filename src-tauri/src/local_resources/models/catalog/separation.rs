use crate::local_resources::models::ModelDescriptor;

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
