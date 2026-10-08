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
