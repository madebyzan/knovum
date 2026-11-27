use super::super::{
    snapshots::SnapshotKind,
    sources::{self, HubSource},
};

#[test]
fn fixed_hub_urls_preserve_revision_and_encode_nested_file_names() {
    let source = sources::for_snapshot(SnapshotKind::ImageSegmentation(
        super::super::catalog::SegmentationModel::IsnetAnime,
    ))
    .unwrap();
    assert_eq!(source.url("nested/a b.onnx").as_str(), "https://huggingface.co/skytnt/anime-seg/resolve/493cb60893f47441b26ec4fb9a306bce9e342982/nested/a%20b.onnx");
    let source = sources::for_snapshot(SnapshotKind::TextEmbedding).unwrap();
    let url = source.url("nested/a+b.json");
    assert_eq!(url.path(), "/api/v1/models/Qwen/Qwen3-Embedding-0.6B/repo");
    assert_eq!(
        url.query_pairs().collect::<Vec<_>>(),
        vec![
            (
                "Revision".into(),
                "dd3e37eb506b3addbd930037796e25d349ae9010".into()
            ),
            ("FilePath".into(), "nested/a+b.json".into()),
        ]
    );
}

#[test]
fn catalog_owns_one_pinned_source_per_snapshot() {
    let transcription = sources::for_snapshot(SnapshotKind::Whisper(
        super::super::catalog::WhisperVariant::Small,
    ));
    let embedding = sources::for_snapshot(SnapshotKind::TextEmbedding).unwrap();
    assert!(transcription.is_none());
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
    let segmentation = super::super::catalog::SegmentationModel::ALL
        .into_iter()
        .map(SnapshotKind::ImageSegmentation);
    for snapshot in std::iter::once(SnapshotKind::TextEmbedding).chain(segmentation) {
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
        (SnapshotKind::VoxCpm2Bf16, "voxcpm2-bf16.gguf"),
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
fn separation_snapshots_pin_the_htdemucs_export_and_its_licence() {
    use super::super::catalog::SeparationVariant;
    for (variant, weights, bytes) in [
        (
            SeparationVariant::HtDemucs,
            "htdemucs-q8_0.gguf",
            61_941_855,
        ),
        (
            SeparationVariant::HtDemucsF16,
            "htdemucs-f16.gguf",
            84_060_255,
        ),
    ] {
        let snapshot = SnapshotKind::Separation(variant);
        assert!(sources::for_snapshot(snapshot).is_none());
        assert_eq!(
            snapshot.files().iter().map(|f| f.path).collect::<Vec<_>>(),
            vec![weights, "LICENSE"]
        );
        assert_eq!(snapshot.total_bytes(), bytes);
        for kind in snapshot.models() {
            let url = super::super::catalog::direct_url(*kind);
            assert!(url.starts_with("https://"));
            assert!(
                url.contains("/73ce4162f6cce370dd533c6782b772135c1b2a8e/")
                    || url.ends_with("/LICENSE")
            );
            assert_eq!(super::super::catalog::descriptor(*kind).snapshot, snapshot);
        }
    }
}

#[test]
fn forced_alignment_uses_the_inference_f16_package() {
    let snapshot = SnapshotKind::SpeechAlignment;
    assert!(sources::for_snapshot(snapshot).is_none());
    assert_eq!(
        snapshot.files().iter().map(|f| f.path).collect::<Vec<_>>(),
        vec!["qwen3-forced-aligner-0.6b-f16.gguf", "LICENSE", "README.md"]
    );
    assert_eq!(snapshot.name(), "qwen3-forced-aligner-0.6b-f16-bundled");
    assert_eq!(snapshot.total_bytes(), 1_840_118_458);
}

#[test]
fn huggingface_resolve_url_builds_standard_resolve_path_with_default_endpoint() {
    let url = sources::huggingface_resolve_url(
        "PaddlePaddle/PP-OCRv6_small_det_onnx",
        "28fe5895c24fd108c19eb3e8479f4ab385fbfc62",
        "inference.onnx",
    );
    assert_eq!(
        url,
        "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_det_onnx/resolve/28fe5895c24fd108c19eb3e8479f4ab385fbfc62/inference.onnx"
    );
}
