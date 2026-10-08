use super::*;
#[test]
fn model_preferences_are_strict_and_belong_to_their_capability() {
    for (group, model) in [
        (Group::Transcription, LocalModelId::WhisperTiny),
        (Group::SpeechSynthesis, LocalModelId::VoxCpm2),
        (Group::SpeechSynthesis, LocalModelId::VoxCpm2Q8),
        (Group::ImageSegmentation, LocalModelId::U2net),
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
fn alignment_defaults_to_the_smaller_native_model_and_keeps_full_precision_selectable() {
    assert_eq!(
        Group::SpeechAlignment.default_model(),
        LocalModelId::Qwen3AlignerQ8
    );
    assert!(group(LocalModelId::Qwen3Aligner) == Some(Group::SpeechAlignment));
}
