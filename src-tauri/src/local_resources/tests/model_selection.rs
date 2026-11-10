use super::*;
#[test]
fn model_preferences_are_strict_and_belong_to_their_capability() {
    for (group, model) in [
        (Group::Transcription, LocalModelId::WhisperTiny),
        (Group::SpeechSynthesis, LocalModelId::VoxCpm2),
        (Group::SpeechSynthesis, LocalModelId::VoxCpm2Q8),
        (Group::ImageSegmentation, LocalModelId::Rmbg14),
        (Group::ImageUpscaling, LocalModelId::RealEsrganX4plus),
        (Group::ImageInpainting, LocalModelId::Migan),
        (Group::ImageInpainting, LocalModelId::Lama),
        (Group::ImageSelection, LocalModelId::EfficientSamS),
        (Group::ImageDepth, LocalModelId::DepthAnythingV2SmallUint8),
        (Group::SpeechAlignment, LocalModelId::Qwen3AlignerQ8),
        (Group::SpeechSeparation, LocalModelId::Spleeter2StemsFull),
        (
            Group::SpeechDiarization,
            LocalModelId::SherpaDiarizationFull,
        ),
    ] {
        let raw = serde_json::to_value(Selection {
            schema_version: 1,
            value: model,
        })
        .unwrap();
        assert_eq!(read_selection(raw.clone(), group).unwrap(), model);
        let other = if group == Group::Transcription {
            Group::SpeechSynthesis
        } else {
            Group::Transcription
        };
        assert!(read_selection(raw, other).is_err());
    }
    assert!(read_selection(
        serde_json::json!({"schemaVersion": 0, "value": "whisperTiny"}),
        Group::Transcription
    )
    .is_err());
    assert!(read_selection(
        serde_json::json!({"schemaVersion": 1, "value": "mossNano", "unknown": true}),
        Group::SpeechSynthesis
    )
    .is_err());
    assert!(group(LocalModelId::Qwen3Embedding).is_none());
}

#[test]
fn alignment_defaults_to_the_smaller_compiled_model_and_keeps_full_precision_selectable() {
    assert_eq!(
        Group::SpeechAlignment.default_model(),
        LocalModelId::Qwen3AlignerQ8
    );
    assert!(group(LocalModelId::Qwen3Aligner) == Some(Group::SpeechAlignment));
}

#[test]
fn an_image_capability_resolves_through_the_choice_its_models_belong_to() {
    for model in crate::local_resources::model_catalog::MODELS
        .iter()
        .copied()
    {
        if let Some(choice) = image_group(model.capability()) {
            assert!(
                group(model) == Some(choice),
                "{model:?} is chosen where its capability is resolved"
            );
        }
    }
}

#[test]
fn retired_segmentation_selections_migrate_to_birefnet_default() {
    for legacy in ["u2netp", "silueta", "u2netHumanSeg", "u2net"] {
        let raw = serde_json::json!({"schemaVersion": 1, "value": legacy});
        assert_eq!(
            read_selection(raw, Group::ImageSegmentation)
                .unwrap_or(Group::ImageSegmentation.default_model()),
            LocalModelId::BirefnetLite
        );
    }
}
