use super::*;

#[test]
fn required_transcription_follows_the_selected_model() {
    let variants = [
        LocalModelId::WhisperSmall,
        LocalModelId::WhisperTiny,
        LocalModelId::WhisperBase,
        LocalModelId::WhisperMedium,
        LocalModelId::WhisperMediumFull,
        LocalModelId::WhisperLargeV3TurboQ8,
        LocalModelId::WhisperLargeV3Turbo,
        LocalModelId::WhisperLargeV3,
    ];
    for selected in variants {
        assert!(LocalModelId::Qwen3Embedding.is_essential(selected));
        for model in variants {
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
    for model in [
        LocalModelId::WhisperTiny,
        LocalModelId::WhisperBase,
        LocalModelId::WhisperSmall,
        LocalModelId::WhisperMedium,
        LocalModelId::WhisperMediumFull,
        LocalModelId::WhisperLargeV3TurboQ8,
        LocalModelId::WhisperLargeV3Turbo,
        LocalModelId::WhisperLargeV3,
    ] {
        let parts = model.transcription_models();
        assert!(parts.len() >= 2);
        assert!(parts == model.preparation().models().unwrap());
        assert!(parts
            .iter()
            .all(|part| catalog::descriptor(*part).snapshot == model.snapshot().unwrap()));
    }
}

#[test]
fn segmentation_choices_have_distinct_snapshots_and_frozen_execution_identities() {
    let default = LocalModelId::BirefnetLite;
    let snapshot = default.snapshot().unwrap();
    assert_eq!(snapshot.total_bytes(), 224_005_088);
    assert!(default.preparation().models().unwrap() == snapshot.models());
    let choices = MODELS
        .iter()
        .copied()
        .filter(|model| model.capability() == LocalModelCapability::ImageSegmentation)
        .collect::<Vec<_>>();
    assert_eq!(
        choices.len(),
        catalog::SegmentationModel::ALL.len(),
        "every background-removal model is offered once"
    );
    let mut spec_ids = std::collections::HashSet::new();
    let mut snapshots = std::collections::HashSet::new();
    for model in choices {
        assert!(super::super::model_selection::is_selectable(model));
        let spec_id = model.image_segmentation_spec().unwrap();
        let file = catalog::descriptor(model.snapshot().unwrap().models()[0]);
        assert!(spec_id.contains(file.sha256));
        assert!(spec_ids.insert(spec_id));
        assert!(snapshots.insert(model.snapshot().unwrap()));
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
    for model in [
        compact,
        LocalModelId::SherpaDiarizationCampplus,
        full,
        LocalModelId::SherpaDiarizationEres2netLarge,
    ] {
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
    let bf16 = LocalModelId::VoxCpm2Bf16;
    let q8 = LocalModelId::VoxCpm2Q8;
    assert_ne!(full.snapshot(), q8.snapshot());
    assert_ne!(bf16.snapshot(), q8.snapshot());
    assert_eq!(q8.snapshot().unwrap().total_bytes(), 2_955_019_717);
    assert!(q8.snapshot().unwrap().total_bytes() < full.snapshot().unwrap().total_bytes());
    for model in [full, bf16, q8] {
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
fn image_choices_prepare_their_complete_snapshot_without_recursion() {
    let choices = [
        LocalModelId::RealEsrganGeneralX4v3,
        LocalModelId::RealEsrganX4plus,
        LocalModelId::Lama,
        LocalModelId::MobileSam,
        LocalModelId::Sam2HieraTiny,
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
        LocalModelId::MobileSam.snapshot().unwrap().total_bytes()
            < LocalModelId::Sam2HieraTiny
                .snapshot()
                .unwrap()
                .total_bytes()
    );
}

#[test]
fn a_hardware_figure_describes_the_machine_and_never_withholds_a_model() {
    use crate::local_resources::model_metadata::{metadata, LocalModelMemory};
    const GIB: u64 = 1024 * 1024 * 1024;

    // A model that stays within the ordinary budget never produces an advisory, however small the
    // machine is.
    assert_eq!(LocalModelMemory::Any.advisory_bytes(Some(GIB)), None);
    // A qualified model reports its figure only to a machine that has less, and says nothing about
    // a machine we could not measure.
    let qualified = LocalModelMemory::QualifiedOnGib(32);
    assert_eq!(qualified.advisory_bytes(Some(16 * GIB)), Some(32 * GIB));
    assert_eq!(qualified.advisory_bytes(Some(32 * GIB)), None);
    assert_eq!(qualified.advisory_bytes(Some(64 * GIB)), None);
    assert_eq!(qualified.advisory_bytes(None), None);

    // Every model states what it needs, so a larger one added later cannot arrive without an
    // answer. VoxCPM2 claims the machine it was qualified on; the BiRefNet and BEN2 segmentation
    // models claim the smallest machine whose budget admits their reserved segmentation peak.
    for &model in MODELS {
        let declared = matches!(metadata(model).memory, LocalModelMemory::QualifiedOnGib(_));
        assert_eq!(
            declared,
            matches!(
                model,
                LocalModelId::VoxCpm2
                    | LocalModelId::VoxCpm2Bf16
                    | LocalModelId::VoxCpm2Q8
                    | LocalModelId::BirefnetLite
                    | LocalModelId::BirefnetGeneral
                    | LocalModelId::BirefnetPortrait
                    | LocalModelId::Ben2Base
            ),
            "{model:?} changed its machine-level claim"
        );
    }
}
