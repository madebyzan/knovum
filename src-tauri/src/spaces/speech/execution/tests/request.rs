use super::*;

#[test]
fn saved_designed_voice_reuses_the_selected_recording_without_redesigning() {
    let voice = VoiceExecution {
        revision_id: "selected-design".into(),
        source_kind: "designed".into(),
        engine_id: "voxcpm2".into(),
        model_spec_id: "voxcpm2-2b-gguf-orig-v1".into(),
        language: "en".into(),
        preset_voice_id: None,
        reference_asset_id: Some("/managed/selected.wav".into()),
        reference_source_revision: Some(format!("sha256:{}", "a".repeat(64))),
        reference_text: Some("The selected sample text.".into()),
        design_description: Some("Warm and calm".into()),
        synthesis_config: "{}".into(),
    };
    let reference = AudioFileReference {
        path: "/managed/selected.wav".into(),
        media_type: "audio/wav".into(),
        byte_length: 160044,
        sha256: "a".repeat(64),
        sample_rate: 16000,
        channels: 1,
        sample_format: "pcm_s16le".into(),
    };
    let revision = frozen_voice_revision(&voice, Some(reference.clone())).unwrap();
    assert!(matches!(revision.source_kind, VoiceSourceKind::Reference));
    assert_eq!(revision.reference_audio.unwrap().sha256, reference.sha256);
    assert_eq!(revision.reference_text, voice.reference_text);
    assert!(revision.design_prompt.is_none());
    assert!(revision.speaker_id.is_none());
}

#[test]
fn reference_voice_with_design_prompt_preserves_prompt_guidance() {
    let voice = VoiceExecution {
        revision_id: "selected-reference".into(),
        source_kind: "reference".into(),
        engine_id: "voxcpm2".into(),
        model_spec_id: "voxcpm2-2b-gguf-orig-v1".into(),
        language: "zh-CN".into(),
        preset_voice_id: None,
        reference_asset_id: Some("/managed/ref.wav".into()),
        reference_source_revision: Some(format!("sha256:{}", "b".repeat(64))),
        reference_text: None,
        design_description: Some("沉稳磁性".into()),
        synthesis_config: "{}".into(),
    };
    let reference = AudioFileReference {
        path: "/managed/ref.wav".into(),
        media_type: "audio/wav".into(),
        byte_length: 160044,
        sha256: "b".repeat(64),
        sample_rate: 16000,
        channels: 1,
        sample_format: "pcm_s16le".into(),
    };
    let revision = frozen_voice_revision(&voice, Some(reference)).unwrap();
    assert!(matches!(revision.source_kind, VoiceSourceKind::Reference));
    assert_eq!(revision.design_prompt.as_deref(), Some("沉稳磁性"));
}

#[test]
fn transcribed_reference_voice_is_cloned_as_spoken_without_style_guidance() {
    let voice = VoiceExecution {
        revision_id: "transcribed-reference".into(),
        source_kind: "reference".into(),
        engine_id: "voxcpm2".into(),
        model_spec_id: "voxcpm2-2b-gguf-orig-v1".into(),
        language: "zh-CN".into(),
        preset_voice_id: None,
        reference_asset_id: Some("/managed/ref.wav".into()),
        reference_source_revision: Some(format!("sha256:{}", "c".repeat(64))),
        reference_text: Some("参考录音的原文。".into()),
        design_description: Some("沉稳磁性".into()),
        synthesis_config: "{}".into(),
    };
    let reference = AudioFileReference {
        path: "/managed/ref.wav".into(),
        media_type: "audio/wav".into(),
        byte_length: 160044,
        sha256: "c".repeat(64),
        sample_rate: 16000,
        channels: 1,
        sample_format: "pcm_s16le".into(),
    };
    let revision = frozen_voice_revision(&voice, Some(reference)).unwrap();
    assert_eq!(revision.reference_text, voice.reference_text);
    assert!(revision.design_prompt.is_none());
}
