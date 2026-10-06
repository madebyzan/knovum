use serde::Serialize;

use super::{catalog, snapshots::SnapshotKind};
use crate::local_resources::preferences::ResourceDownloadRegion;

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

pub(super) fn for_snapshot(snapshot: SnapshotKind, region: ResourceDownloadRegion) -> HubSource {
    let (repository, revision, china_revision) = match snapshot {
        SnapshotKind::FasterWhisper => (
            "Systran/faster-whisper-small",
            "536b0662742c02347bc0e980a01041f333bce120",
            Some("ace8b2ad9dee031c53b6371f6c3c918b5e4f1db9"),
        ),
        SnapshotKind::MlxWhisper => (
            "mlx-community/whisper-small-mlx",
            "45f3915923c7a79a5a5b5a7d909d39aeb0e5630e",
            Some("096cc7071475c5a17a5fc315a823bc948f26d430"),
        ),
        SnapshotKind::TextEmbedding => (
            "Qwen/Qwen3-Embedding-0.6B",
            "97b0c614be4d77ee51c0cef4e5f07c00f9eb65b3",
            Some("dd3e37eb506b3addbd930037796e25d349ae9010"),
        ),
        SnapshotKind::VoxCpm2 => (
            "OpenBMB/VoxCPM2",
            "32279effe8c19989596f05d353d1447f51d9e915",
            Some("519da200db2bc31a0395635fc7042653b9c4c862"),
        ),
        SnapshotKind::SpeechAlignment => (
            "Qwen/Qwen3-ForcedAligner-0.6B",
            catalog::QWEN3_FORCED_ALIGNER_REVISION,
            Some("cf1c50164ea3ac48240d12bef5ead74aee0720cc"),
        ),
        SnapshotKind::ImageSegmentation => (
            "tomjackson2023/rembg",
            "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
            None,
        ),
        SnapshotKind::SpeechSeparation => (
            "csukuangfj/sherpa-onnx-spleeter-2stems-fp16",
            "93ba771920ade509f8cbd6825b1a90856c797e08",
            None,
        ),
    };
    match (region, china_revision) {
        (ResourceDownloadRegion::China, Some(revision)) => HubSource::ModelScope {
            repository,
            revision,
        },
        (ResourceDownloadRegion::China, None) => HubSource::HuggingFace {
            repository,
            revision,
            endpoint: "https://hf-mirror.com",
        },
        (ResourceDownloadRegion::Global, _) => HubSource::HuggingFace {
            repository,
            revision,
            endpoint: "https://huggingface.co",
        },
    }
}
