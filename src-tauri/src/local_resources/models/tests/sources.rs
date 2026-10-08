use super::super::{
    snapshots::SnapshotKind,
    sources::{self, HubSource},
};

#[test]
fn catalog_owns_one_pinned_source_per_snapshot() {
    let transcription = sources::for_snapshot(SnapshotKind::FasterWhisper).unwrap();
    let embedding = sources::for_snapshot(SnapshotKind::TextEmbedding).unwrap();
    assert!(matches!(
        transcription,
        HubSource::HuggingFace {
            endpoint: "https://huggingface.co",
            ..
        }
    ));
    assert!(matches!(
        embedding,
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
        SnapshotKind::SpeechAlignment,
        SnapshotKind::ImageSegmentation,
        SnapshotKind::ImageSegmentationQuality,
        SnapshotKind::SpeechSeparation,
        SnapshotKind::SpeechSeparationFull,
    ] {
        let revision = match sources::for_snapshot(snapshot).unwrap() {
            HubSource::HuggingFace { revision, .. } | HubSource::ModelScope { revision, .. } => {
                revision
            }
        };
        assert_eq!(revision.len(), 40);
        assert!(revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
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

#[test]
fn moss_joint_snapshot_is_pinned_and_downloads_without_hub_runtime() {
    use super::super::catalog::ModelKind;
    let snapshot = SnapshotKind::MossNano;
    assert!(sources::for_snapshot(snapshot).is_none());
    assert_eq!(snapshot.models().len(), 18);
    for kind in snapshot.models() {
        let ModelKind::MossNanoPart(part) = kind else {
            panic!("MOSS catalog part")
        };
        let file = part.descriptor();
        assert_eq!(file.snapshot, snapshot);
        assert_eq!(file.sha256.len(), 64);
        assert!(part
            .url()
            .starts_with("https://huggingface.co/OpenMOSS-Team/"));
    }
    assert!(snapshot.total_bytes() < 800_000_000);
}

#[test]
fn compiled_voice_snapshot_pins_weights_and_notices_without_a_hub_runtime() {
    for (snapshot, weight) in [
        (SnapshotKind::VoxCpm2, "voxcpm2-orig.gguf"),
        (SnapshotKind::VoxCpm2Q8, "voxcpm2-q8_0.gguf"),
    ] {
        assert!(sources::for_snapshot(snapshot).is_none());
        assert_eq!(snapshot.models().len(), 3);
        assert!(snapshot.files().iter().any(|file| file.path == weight));
        for kind in snapshot.models() {
            assert!(super::super::catalog::direct_url(*kind).starts_with("https://"));
            assert_eq!(super::super::catalog::descriptor(*kind).snapshot, snapshot);
        }
    }
}

#[test]
fn forced_alignment_uses_the_complete_compiled_transformers_checkpoint() {
    use super::super::catalog;
    let snapshot = SnapshotKind::SpeechAlignment;
    assert!(matches!(
        sources::for_snapshot(snapshot).unwrap(),
        HubSource::HuggingFace {
            repository: "Qwen/Qwen3-ForcedAligner-0.6B-hf",
            revision: catalog::QWEN3_FORCED_ALIGNER_REVISION,
            endpoint: "https://huggingface.co"
        }
    ));
    let files = snapshot.files();
    let paths = files
        .iter()
        .map(|file| file.path)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        paths,
        std::collections::HashSet::from([
            "chat_template.jinja",
            "config.json",
            "model.safetensors",
            "processor_config.json",
            "tokenizer.json",
            "tokenizer_config.json",
        ])
    );
    assert_eq!(snapshot.name(), "qwen3-forced-aligner-0.6b-hf");
    assert_eq!(snapshot.total_bytes(), 1_847_225_867);
}
