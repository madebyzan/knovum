use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

pub(crate) const QWEN3_FORCED_ALIGNER_REVISION: &str = "09fe073ba154561f4474162e8bd4ab233a848eca";
pub(crate) const QWEN3_FORCED_ALIGNER_WEIGHTS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechAlignment,
    file: "qwen3-forced-aligner-0.6b-f16.gguf",
    sha256: "d0a96fb96de17686766688b622394877ee5e89b193be7f53de8715d99151a000",
    download_size: 1_840_097_696,
};

pub(crate) const QWEN3_ALIGNER_Q8_REVISION: &str = "09fe073ba154561f4474162e8bd4ab233a848eca";
pub(crate) const QWEN3_ALIGNER_Q8_MODELS: &[super::ModelKind] = &[
    super::ModelKind::Qwen3AlignerQ8Weights,
    super::ModelKind::Qwen3AlignerQ8License,
    super::ModelKind::Qwen3AlignerQ8Readme,
];

pub(super) const QWEN3_ALIGNER_Q8_WEIGHTS: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechAlignmentQ8,
    file: "qwen3-forced-aligner-0.6b-q8_0.gguf",

    sha256: "75209490b11cec2b0db749ca5f4ff92266f58efd30f7fd04d9eb2a3ac9cc929f",
    download_size: 1_129_966_496,
};
pub(super) const QWEN3_ALIGNER_Q8_LICENSE: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechAlignmentQ8,
    file: "LICENSE",

    sha256: "a44a6081c73ad75f0255bb2bb5cab74ef1829565a895a24e53a4f11290ab7655",
    download_size: 11_343,
};
pub(super) const QWEN3_ALIGNER_Q8_README: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechAlignmentQ8,
    file: "README.md",

    sha256: "30de8315ece609b239c719539336b99188a2a92aef89a1fdcd60b11e40abd063",
    download_size: 9_419,
};
pub(super) const QWEN3_ALIGNER_F16_LICENSE: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechAlignment,
    ..QWEN3_ALIGNER_Q8_LICENSE
};
pub(super) const QWEN3_ALIGNER_F16_README: ModelDescriptor = ModelDescriptor {
    snapshot: SnapshotKind::SpeechAlignment,
    ..QWEN3_ALIGNER_Q8_README
};

pub(super) fn direct_url(kind: super::ModelKind) -> &'static str {
    match kind {
        super::ModelKind::Qwen3ForcedAlignerWeights => "https://huggingface.co/audio-cpp/audio.cpp-gguf/resolve/09fe073ba154561f4474162e8bd4ab233a848eca/Qwen3-ForcedAligner-0.6B-GGUF/qwen3-forced-aligner-0.6b-f16.gguf",
        super::ModelKind::Qwen3AlignerQ8Weights => "https://huggingface.co/audio-cpp/audio.cpp-gguf/resolve/09fe073ba154561f4474162e8bd4ab233a848eca/Qwen3-ForcedAligner-0.6B-GGUF/qwen3-forced-aligner-0.6b-q8_0.gguf",
        super::ModelKind::Qwen3AlignerQ8License | super::ModelKind::Qwen3ForcedAlignerLicense => "https://raw.githubusercontent.com/QwenLM/Qwen3-ASR/7c6daf77a2421100f5fb066495372c00129d39ff/LICENSE",
        super::ModelKind::Qwen3AlignerQ8Readme | super::ModelKind::Qwen3ForcedAlignerReadme => "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B-hf/resolve/c07281df297b9905d24a508279258cccf987a064/README.md",
        _ => unreachable!("Compiled alignment owns exactly three frozen files"),
    }
}
