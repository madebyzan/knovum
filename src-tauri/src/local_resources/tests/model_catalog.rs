use super::*;

#[test]
fn required_transcription_follows_the_selected_model() {
    for selected in [
        LocalModelId::WhisperSmall,
        LocalModelId::WhisperTiny,
        LocalModelId::WhisperLargeV3,
    ] {
        assert!(LocalModelId::Qwen3Embedding.is_essential(selected));
        for model in [
            LocalModelId::WhisperSmall,
            LocalModelId::WhisperTiny,
            LocalModelId::WhisperLargeV3,
        ] {
            assert_eq!(model.is_essential(selected), model == selected);
        }
    }
}

#[test]
fn fixed_embedding_and_platform_transcription_use_the_same_snapshots_as_node_jobs() {
    assert_eq!(
        LocalModelId::Qwen3Embedding.snapshot(),
        Some(SnapshotKind::TextEmbedding)
    );
    assert!(
        LocalModelId::WhisperSmall.snapshot().unwrap().models() == catalog::TRANSCRIPTION_MODELS
    );
    assert!(matches!(
        LocalModelId::VoxCpm2.preparation(),
        ResourcePreparationTarget::VoiceDesign
    ));
    assert_eq!(
        MODELS
            .iter()
            .filter(|&&id| id == LocalModelId::Qwen3Embedding)
            .count(),
        1
    );
}

#[test]
fn transcription_alternatives_own_complete_distinct_snapshots() {
    let tiny = LocalModelId::WhisperTiny.snapshot().unwrap();
    let large = LocalModelId::WhisperLargeV3.snapshot().unwrap();
    assert_ne!(tiny, large);
    assert!(tiny.total_bytes() < 100_000_000);
    assert!(large.total_bytes() > 3_000_000_000);
    for model in [LocalModelId::WhisperTiny, LocalModelId::WhisperLargeV3] {
        let parts = model.transcription_models();
        assert!(parts.len() >= 2);
        assert!(parts == model.preparation().models().unwrap());
        assert!(parts
            .iter()
            .all(|part| catalog::descriptor(*part).snapshot == model.snapshot().unwrap()));
    }
}

#[test]
fn full_image_model_has_its_own_snapshot_and_frozen_execution_identity() {
    let compact = LocalModelId::U2netp;
    let full = LocalModelId::U2net;
    let snapshot = full.snapshot().unwrap();
    assert_ne!(compact.snapshot(), full.snapshot());
    assert_eq!(snapshot.total_bytes(), 175_997_641);
    assert!(full.preparation().models().unwrap() == snapshot.models());
    assert_ne!(
        compact.image_segmentation_spec().unwrap(),
        full.image_segmentation_spec().unwrap()
    );
    for model in [compact, full] {
        assert!(super::super::model_selection::is_selectable(model));
        let file = catalog::descriptor(model.snapshot().unwrap().models()[0]);
        assert!(model
            .image_segmentation_spec()
            .unwrap()
            .contains(file.sha256));
    }
    assert!(LocalModelId::Qwen3Embedding
        .image_segmentation_spec()
        .is_err());
}

#[test]
fn separation_choices_have_distinct_complete_snapshots_and_execution_identities() {
    let compact = LocalModelId::Spleeter2Stems;
    let full = LocalModelId::Spleeter2StemsFull;
    assert_ne!(compact.snapshot(), full.snapshot());
    assert_ne!(
        compact.separation_spec().unwrap(),
        full.separation_spec().unwrap()
    );
    assert_eq!(full.snapshot().unwrap().total_bytes(), 78_636_679);
    for model in [compact, full] {
        assert!(super::super::model_selection::is_selectable(model));
        assert_eq!(
            LocalModelId::from_separation_spec(model.separation_spec().unwrap()).unwrap(),
            model
        );
        let snapshot = model.snapshot().unwrap();
        assert!(model.preparation().models().unwrap() == snapshot.models());
        assert_eq!(snapshot.models().len(), 2);
        assert!(snapshot
            .models()
            .iter()
            .all(|part| catalog::descriptor(*part).snapshot == snapshot));
    }
    assert!(LocalModelId::WhisperSmall.separation_spec().is_err());
}

#[test]
fn diarization_choices_own_complete_snapshots_without_a_special_installer() {
    let compact = LocalModelId::SherpaDiarization;
    let full = LocalModelId::SherpaDiarizationFull;
    assert_ne!(compact.snapshot(), full.snapshot());
    assert_eq!(compact.snapshot().unwrap().total_bytes(), 41_146_685);
    assert_eq!(full.snapshot().unwrap().total_bytes(), 45_599_092);
    for model in [compact, full] {
        let variant = model.diarization_variant().unwrap();
        assert_eq!(
            LocalModelId::from_diarization_spec(variant.spec_id()).unwrap(),
            model
        );
        assert!(super::super::model_selection::is_selectable(model));
        let snapshot = model.snapshot().unwrap();
        assert!(snapshot.models() == model.preparation().models().unwrap());
        assert_eq!(snapshot.models().len(), 4);
        assert!(super::super::models::sources::for_snapshot(snapshot).is_none());
        for kind in snapshot.models() {
            let file = catalog::descriptor(*kind);
            assert_eq!(file.snapshot, snapshot);
            assert_eq!(file.sha256.len(), 64);
            let catalog::ModelKind::DiarizationPart(part) = kind else {
                panic!("Diarization snapshots own diarization parts")
            };
            assert!(part.url().starts_with("https://"));
        }
    }
    assert!(LocalModelId::from_diarization_spec("obsolete-model").is_err());
    assert!(LocalModelId::MossNano.diarization_variant().is_err());
}

#[test]
fn alignment_choices_freeze_model_identity_independently_of_execution_device() {
    for model in [LocalModelId::Qwen3Aligner, LocalModelId::Qwen3AlignerQ8] {
        assert_eq!(
            LocalModelId::from_alignment_spec(model.alignment_spec().unwrap()).unwrap(),
            model
        );
        assert!(super::super::model_selection::is_selectable(model));
        let snapshot = model.snapshot().unwrap();
        assert!(snapshot.models() == model.preparation().models().unwrap());
        assert!(snapshot
            .models()
            .iter()
            .all(|kind| catalog::descriptor(*kind).snapshot == snapshot));
    }
    let q8 = LocalModelId::Qwen3AlignerQ8.snapshot().unwrap();
    assert!(q8.total_bytes() < LocalModelId::Qwen3Aligner.snapshot().unwrap().total_bytes());
    assert!(super::super::models::sources::for_snapshot(q8).is_none());
    assert!(LocalModelId::from_alignment_spec("obsolete-alignment").is_err());
}

#[test]
fn voxcpm2_choices_freeze_distinct_complete_snapshots() {
    let full = LocalModelId::VoxCpm2;
    let q8 = LocalModelId::VoxCpm2Q8;
    assert_ne!(full.snapshot(), q8.snapshot());
    assert_eq!(q8.snapshot().unwrap().total_bytes(), 2_955_019_717);
    assert!(q8.snapshot().unwrap().total_bytes() < full.snapshot().unwrap().total_bytes());
    for model in [full, q8] {
        assert_eq!(
            LocalModelId::from_voxcpm2_spec(model.voxcpm2_spec().unwrap()).unwrap(),
            model
        );
        assert!(super::super::model_selection::is_selectable(model));
        let snapshot = model.snapshot().unwrap();
        assert!(
            model.voxcpm2_models().unwrap() == snapshot.models(),
            "Selected model files differ from the snapshot for {model:?}"
        );
        assert!(snapshot
            .models()
            .iter()
            .all(|kind| catalog::descriptor(*kind).snapshot == snapshot));
    }
}

#[test]
fn catalog_availability_is_supplied_by_application_composition() {
    assert_eq!(
        availability_reason(LocalModelId::VoxCpm2, |_| Ok(false)).unwrap(),
        Some("memory")
    );
    assert_eq!(
        availability_reason(LocalModelId::VoxCpm2Q8, |_| Ok(true)).unwrap(),
        None
    );
    assert_eq!(
        availability_reason(LocalModelId::WhisperSmall, |_| {
            panic!("non-voice models do not consult AI policy")
        })
        .unwrap(),
        None
    );
}

#[test]
fn image_choices_prepare_their_complete_snapshot_without_recursion() {
    let choices = [
        LocalModelId::RealEsrganGeneralX4v3,
        LocalModelId::RealEsrganX4plus,
        LocalModelId::Migan,
        LocalModelId::EfficientSamTi,
        LocalModelId::EfficientSamS,
        LocalModelId::DepthAnythingV2SmallUint8,
        LocalModelId::DepthAnythingV2Small,
    ];
    let mut identities = std::collections::HashSet::new();
    for model in choices {
        let snapshot = model.snapshot().unwrap();
        assert!(identities.insert(snapshot.digest()));
        assert!(snapshot.models() == model.preparation().models().unwrap());
        assert!(snapshot
            .models()
            .iter()
            .all(|part| catalog::descriptor(*part).snapshot == snapshot));
        assert!(
            snapshot.models().len() >= 2,
            "Weights and license are one snapshot"
        );
        assert!(super::super::models::sources::for_snapshot(snapshot).is_none());
    }
    assert!(
        LocalModelId::EfficientSamTi
            .snapshot()
            .unwrap()
            .total_bytes()
            < LocalModelId::EfficientSamS
                .snapshot()
                .unwrap()
                .total_bytes()
    );
}
