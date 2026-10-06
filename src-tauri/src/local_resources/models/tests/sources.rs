use super::super::{
    snapshots::SnapshotKind,
    sources::{self, HubSource},
};
use crate::local_resources::preferences::ResourceDownloadRegion;

#[test]
fn regions_choose_one_pinned_source_without_transport_fallback() {
    let global = sources::for_snapshot(SnapshotKind::TextEmbedding, ResourceDownloadRegion::Global);
    let china = sources::for_snapshot(SnapshotKind::TextEmbedding, ResourceDownloadRegion::China);
    assert!(matches!(
        global,
        HubSource::HuggingFace {
            endpoint: "https://huggingface.co",
            ..
        }
    ));
    assert!(matches!(
        china,
        HubSource::ModelScope {
            repository: "Qwen/Qwen3-Embedding-0.6B",
            revision: "dd3e37eb506b3addbd930037796e25d349ae9010"
        }
    ));
}

#[test]
fn all_sources_pin_full_revisions_and_bundle_files_have_one_owner() {
    for snapshot in [
        SnapshotKind::FasterWhisper,
        SnapshotKind::MlxWhisper,
        SnapshotKind::TextEmbedding,
        SnapshotKind::VoxCpm2,
        SnapshotKind::SpeechAlignment,
        SnapshotKind::ImageSegmentation,
        SnapshotKind::SpeechSeparation,
    ] {
        for region in [
            ResourceDownloadRegion::Global,
            ResourceDownloadRegion::China,
        ] {
            let revision = match sources::for_snapshot(snapshot, region) {
                HubSource::HuggingFace { revision, .. }
                | HubSource::ModelScope { revision, .. } => revision,
            };
            assert_eq!(revision.len(), 40);
            assert!(revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
        }
        for kind in snapshot.models() {
            assert_eq!(super::super::catalog::descriptor(*kind).snapshot, snapshot);
        }
        let files = snapshot.files();
        assert_eq!(
            files
                .iter()
                .map(|file| file.path)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            files.len()
        );
        assert!(snapshot.total_bytes() > 0);
        assert_eq!(snapshot.digest().len(), 64);
    }
}
