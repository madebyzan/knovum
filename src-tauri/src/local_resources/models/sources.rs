use super::snapshots::SnapshotKind;

#[derive(Clone, Debug)]
pub(crate) enum HubSource {
    HuggingFace {
        repository: &'static str,
        revision: &'static str,
        endpoint: &'static str,
    },
    ModelScope {
        repository: &'static str,
        revision: &'static str,
    },
}

impl HubSource {
    pub(super) fn url(&self, file: &str) -> reqwest::Url {
        let (mut url, repository) = match self {
            Self::HuggingFace {
                endpoint,
                repository,
                ..
            } => (
                reqwest::Url::parse(endpoint).expect("Catalog owns a valid hub endpoint"),
                repository,
            ),
            Self::ModelScope { repository, .. } => (
                reqwest::Url::parse("https://modelscope.cn/api/v1/models/")
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
        | SnapshotKind::VoxCpm2Q8
        | SnapshotKind::SpeechAlignment
        | SnapshotKind::SpeechAlignmentQ8
        | SnapshotKind::Diarization(_) => return None,
        SnapshotKind::Whisper(_) => return None,
        SnapshotKind::TextEmbedding => HubSource::ModelScope {
            repository: "Qwen/Qwen3-Embedding-0.6B",
            revision: "dd3e37eb506b3addbd930037796e25d349ae9010",
        },
        SnapshotKind::ImageSegmentation | SnapshotKind::ImageSegmentationQuality => {
            HubSource::HuggingFace {
                repository: "tomjackson2023/rembg",
                revision: "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
                endpoint: "https://huggingface.co",
            }
        }
        SnapshotKind::SpeechSeparationFull => HubSource::HuggingFace {
            repository: "csukuangfj/sherpa-onnx-spleeter-2stems",
            revision: "7001ba316a615cacddb3f9ef3ec416661a277e26",
            endpoint: "https://huggingface.co",
        },
        SnapshotKind::SpeechSeparation => HubSource::HuggingFace {
            repository: "csukuangfj/sherpa-onnx-spleeter-2stems-fp16",
            revision: "93ba771920ade509f8cbd6825b1a90856c797e08",
            endpoint: "https://huggingface.co",
        },
    })
}
