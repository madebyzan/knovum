use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

pub(crate) const QWEN3_FORCED_ALIGNER_REVISION: &str = "c7cbfc2048c462b0d63a45797104fc9db3ad62b7";

macro_rules! qwen3_forced_aligner_descriptor {
    ($name:ident, $file:literal, $env:literal, $sha:literal, $size:literal) => {
        pub(crate) const $name: ModelDescriptor = ModelDescriptor {
            snapshot: SnapshotKind::SpeechAlignment,
            file: $file,
            env_key: $env,
            sha256: $sha,
            download_size: $size,
        };
    };
}

qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE,
    "chat_template.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_CHAT_TEMPLATE_PATH",
    "75a8cfca24f00de72d796fbfed6858fc9614ef3dabd8696684cc3bc03a9c58ff",
    1_161
);

qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_CONFIG,
    "config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_CONFIG_PATH",
    "d616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09",
    5_982
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_GENERATION_CONFIG,
    "generation_config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_GENERATION_CONFIG_PATH",
    "948d089b23bca1d214e768d59c4438365665f52ec6d33678f4062206b3fbbb8c",
    115
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_MERGES,
    "merges.txt",
    "KNOVUM_QWEN3_FORCED_ALIGNER_MERGES_PATH",
    "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5",
    1_671_853
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_WEIGHTS,
    "model.safetensors",
    "KNOVUM_QWEN3_FORCED_ALIGNER_WEIGHTS_PATH",
    "47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7",
    1_835_544_544
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_PREPROCESSOR_CONFIG,
    "preprocessor_config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_PREPROCESSOR_CONFIG_PATH",
    "45e120a4eda2c20c5d7f2ea9354e63536bf35e27aa573fb7cdf78017b378770d",
    330
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG,
    "tokenizer_config.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_TOKENIZER_CONFIG_PATH",
    "3ab80063f8511deb9566e6ad438d17b7a6277fcffd52d92854112f19d36bd81c",
    12_666
);
qwen3_forced_aligner_descriptor!(
    QWEN3_FORCED_ALIGNER_VOCABULARY,
    "vocab.json",
    "KNOVUM_QWEN3_FORCED_ALIGNER_VOCABULARY_PATH",
    "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910",
    2_776_833
);
