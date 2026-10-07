use serde::Serialize;

use super::snapshots::SnapshotKind;

#[derive(Clone, Serialize, Debug, schemars::JsonSchema)]
#[serde(tag = "provider", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum HubSource {
    #[schemars(title = "HuggingFaceSource")]
    HuggingFace {
        repository: &'static str,
        revision: &'static str,
        endpoint: &'static str,
    },
    #[schemars(title = "ModelScopeSource")]
    ModelScope {
        repository: &'static str,
        revision: &'static str,
    },
}

pub(super) fn for_snapshot(snapshot: SnapshotKind) -> HubSource {
    match snapshot {
        SnapshotKind::FasterWhisper => HubSource::HuggingFace {
            repository: "Systran/faster-whisper-small",
            revision: "536b0662742c02347bc0e980a01041f333bce120",
            endpoint: "https://huggingface.co",
        },
        SnapshotKind::MlxWhisper => HubSource::HuggingFace {
            repository: "mlx-community/whisper-small-mlx",
            revision: "45f3915923c7a79a5a5b5a7d909d39aeb0e5630e",
            endpoint: "https://huggingface.co",
        },
        SnapshotKind::TextEmbedding => HubSource::ModelScope {
            repository: "Qwen/Qwen3-Embedding-0.6B",
            revision: "dd3e37eb506b3addbd930037796e25d349ae9010",
        },
        SnapshotKind::VoxCpm2 => HubSource::ModelScope {
            repository: "OpenBMB/VoxCPM2",
            revision: "519da200db2bc31a0395635fc7042653b9c4c862",
        },
        SnapshotKind::SpeechAlignment => HubSource::ModelScope {
            repository: "Qwen/Qwen3-ForcedAligner-0.6B",
            revision: "cf1c50164ea3ac48240d12bef5ead74aee0720cc",
        },
        SnapshotKind::ImageSegmentation => HubSource::HuggingFace {
            repository: "tomjackson2023/rembg",
            revision: "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
            endpoint: "https://huggingface.co",
        },
        SnapshotKind::SpeechSeparation => HubSource::HuggingFace {
            repository: "csukuangfj/sherpa-onnx-spleeter-2stems-fp16",
            revision: "93ba771920ade509f8cbd6825b1a90856c797e08",
            endpoint: "https://huggingface.co",
        },
    }
}
