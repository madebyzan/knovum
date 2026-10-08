use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

pub(crate) const QWEN3_FORCED_ALIGNER_REVISION: &str = "c07281df297b9905d24a508279258cccf987a064";

macro_rules! qwen3_forced_aligner_descriptor {
    ($name:ident, $file:literal, $sha:literal, $size:literal) => {
        pub(crate) const $name: ModelDescriptor = ModelDescriptor {
            snapshot: SnapshotKind::SpeechAlignment,
            file: $file,

            sha256: $sha,
            download_size: $size,
        };
    };
}

qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE,
    "chat_template.jinja",
    "59b5ae119bda7c00089bba5f49af775791ea67c7aeb7d2b828cddda9463121bf",
    551
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_CONFIG,
    "config.json",
    "82ac390b36a8f80c5a0c4e202367debd149e5d8c30abe4cf5eed439101b1c5ce",
    248_029
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_WEIGHTS,
    "model.safetensors",
    "00568245ceca5af1991d28562a75fe1ddc9bfeb041c27fda66947ea05c47fb86",
    1_835_545_960
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_PROCESSOR_CONFIG,
    "processor_config.json",
    "bc0b230081b44e629dd5b9045b78495615c1831b4b9f4cffe97bd37e82a6156a",
    487
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_TOKENIZER,
    "tokenizer.json",
    "cadfb9cbd9ff8f4e309075ec6a71b063295ff796afb4b5dbc0fd2873fbb301f2",
    11_429_842
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG,
    "tokenizer_config.json",
    "945e980986de2ca7768f3326bfdbb4fbea3406f972b8ae0be233089f2b253c11",
    998
);

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
pub(super) fn direct_url(kind: super::ModelKind) -> &'static str {
    match kind {
        super::ModelKind::Qwen3AlignerQ8Weights => "https://huggingface.co/audio-cpp/audio.cpp-gguf/resolve/09fe073ba154561f4474162e8bd4ab233a848eca/Qwen3-ForcedAligner-0.6B-GGUF/qwen3-forced-aligner-0.6b-q8_0.gguf",
        super::ModelKind::Qwen3AlignerQ8License => "https://raw.githubusercontent.com/QwenLM/Qwen3-ASR/7c6daf77a2421100f5fb066495372c00129d39ff/LICENSE",
        super::ModelKind::Qwen3AlignerQ8Readme => "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B-hf/resolve/c07281df297b9905d24a508279258cccf987a064/README.md",
        _ => unreachable!("Native alignment owns exactly three frozen files"),
    }
}
