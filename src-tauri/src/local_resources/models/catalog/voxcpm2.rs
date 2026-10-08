use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

const LICENSE_SHA256: &str = "4f10acc209addacfad28293315c74c4cd648f771ee1263a748f1781d1e0265e4";
const LICENSE_SIZE: u64 = 11_298;
const LICENSE_URL: &str =
    "https://raw.githubusercontent.com/OpenBMB/VoxCPM/f0c787f0937dc1c9a8f4f64d9a332d9c5da2e629/LICENSE";
const README_SHA256: &str = "c18d2ffbfb2f0aec770481c967842aee6c200cde65f6b033efd892a849986fda";
const README_SIZE: u64 = 7_939;
const README_URL: &str =
    "https://huggingface.co/openbmb/VoxCPM2/resolve/32279effe8c19989596f05d353d1447f51d9e915/README.md";

pub(crate) const VOXCPM2_WEIGHTS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "voxcpm2-orig.gguf",

    sha256: "13181e01e15d555c9eae7779626eb395c69334f15e948c9ea265ddf37ab50b16",
    download_size: 4_960_716_192,
};
pub(crate) const VOXCPM2_Q8_WEIGHTS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2Q8,
    file: "voxcpm2-q8_0.gguf",

    sha256: "c8e01ab4416011e12a28f24ede298a1aa5ce64b43f8e8aaad53b1e2fe7c96432",
    download_size: 2_955_000_480,
};
pub(crate) const VOXCPM2_LICENSE: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "LICENSE",

    sha256: LICENSE_SHA256,
    download_size: LICENSE_SIZE,
};
pub(crate) const VOXCPM2_README: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2,
    file: "README.md",

    sha256: README_SHA256,
    download_size: README_SIZE,
};
pub(crate) const VOXCPM2_Q8_LICENSE: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2Q8,
    file: "LICENSE",

    sha256: LICENSE_SHA256,
    download_size: LICENSE_SIZE,
};
pub(crate) const VOXCPM2_Q8_README: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::VoxCpm2Q8,
    file: "README.md",

    sha256: README_SHA256,
    download_size: README_SIZE,
};
pub(super) fn direct_url(kind: super::ModelKind) -> &'static str {
    match kind {
        super::ModelKind::VoxCpm2Weights => "https://huggingface.co/audio-cpp/audio.cpp-gguf/resolve/e36610ac69b5262e914a52635324050bee8f1ad2/VoxCPM2-GGUF/voxcpm2-orig.gguf",
        super::ModelKind::VoxCpm2Q8Weights => "https://huggingface.co/audio-cpp/audio.cpp-gguf/resolve/09fe073ba154561f4474162e8bd4ab233a848eca/VoxCPM2-GGUF/voxcpm2-q8_0.gguf",
        super::ModelKind::VoxCpm2License | super::ModelKind::VoxCpm2Q8License => LICENSE_URL,
        super::ModelKind::VoxCpm2Readme | super::ModelKind::VoxCpm2Q8Readme => README_URL,
        _ => unreachable!("VoxCPM2 snapshots own pinned weights, license, and model card"),
    }
}
