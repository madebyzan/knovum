use super::snapshots::SnapshotKind;

/// The default upstream Hugging Face Hub endpoint.
pub(crate) const DEFAULT_HUGGINGFACE_HUB_ENDPOINT: &str = "https://huggingface.co";

/// The default upstream ModelScope Hub endpoint.
pub(crate) const DEFAULT_MODELSCOPE_HUB_ENDPOINT: &str = "https://modelscope.cn/api/v1/models/";

/// Resolves the active Hugging Face Hub endpoint, respecting the standard `HF_ENDPOINT`
/// environment variable override when configured.
pub(crate) fn huggingface_hub_endpoint() -> String {
    std::env::var("HF_ENDPOINT")
        .ok()
        .map(|value| value.trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_HUGGINGFACE_HUB_ENDPOINT.to_string())
}

/// Constructs a Hugging Face Hub resolve URL for an immutable file revision:
/// `{hub_endpoint}/{repository}/resolve/{revision}/{file}`.
pub(crate) fn huggingface_resolve_url(repository: &str, revision: &str, file: &str) -> String {
    let endpoint = huggingface_hub_endpoint();
    format!("{endpoint}/{repository}/resolve/{revision}/{file}")
}

#[derive(Clone, Debug)]
pub(crate) enum HubSource {
    HuggingFace {
        repository: &'static str,
        revision: &'static str,
    },
    ModelScope {
        repository: &'static str,
        revision: &'static str,
    },
}

impl HubSource {
    pub(super) fn url(&self, file: &str) -> reqwest::Url {
        let (mut url, repository) = match self {
            Self::HuggingFace { repository, .. } => (
                reqwest::Url::parse(&huggingface_hub_endpoint())
                    .expect("Catalog owns a valid Hugging Face hub endpoint"),
                repository,
            ),
            Self::ModelScope { repository, .. } => (
                reqwest::Url::parse(DEFAULT_MODELSCOPE_HUB_ENDPOINT)
                    .expect("Static ModelScope endpoint"),
                repository,
            ),
        };
        {
            let mut path = url
                .path_segments_mut()
                .expect("Hub endpoint accepts path segments");
            path.pop_if_empty().extend(repository.split('/'));
            match self {
                Self::HuggingFace { revision, .. } => {
                    path.push("resolve").push(revision).extend(file.split('/'));
                }
                Self::ModelScope { .. } => {
                    path.push("repo");
                }
            }
        }
        if let Self::ModelScope { revision, .. } = self {
            url.query_pairs_mut()
                .append_pair("Revision", revision)
                .append_pair("FilePath", file);
        }
        url
    }
}

pub(crate) fn for_snapshot(snapshot: SnapshotKind) -> Option<HubSource> {
    Some(match snapshot {
        SnapshotKind::Image(_)
        | SnapshotKind::MossNano
        | SnapshotKind::VoxCpm2
        | SnapshotKind::VoxCpm2Bf16
        | SnapshotKind::VoxCpm2Q8
        | SnapshotKind::SpeechAlignment
        | SnapshotKind::SpeechAlignmentQ8
        | SnapshotKind::Separation(_)
        | SnapshotKind::Voice(_)
        | SnapshotKind::Diarization(_) => return None,
        SnapshotKind::Whisper(_) => return None,
        SnapshotKind::TextEmbedding => HubSource::ModelScope {
            repository: "Qwen/Qwen3-Embedding-0.6B",
            revision: "dd3e37eb506b3addbd930037796e25d349ae9010",
        },
        SnapshotKind::ImageSegmentation
        | SnapshotKind::ImageSegmentationSilueta
        | SnapshotKind::ImageSegmentationHuman
        | SnapshotKind::ImageSegmentationQuality => HubSource::HuggingFace {
            repository: "tomjackson2023/rembg",
            revision: "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
        },
    })
}
