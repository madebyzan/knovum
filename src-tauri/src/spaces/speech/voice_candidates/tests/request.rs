use super::*;

fn request() -> DraftVoiceCandidateRequest {
    DraftVoiceCandidateRequest {
        model: super::super::VoiceModel::MossNano,
        request_id: uuid::Uuid::new_v4().to_string(),
        language: "zh-CN".into(),
        text: "你好世界".into(),
        seed: 0,
        source: VoiceCandidateSource::Reference {
            source_path: "/voice.wav".into(),
            source_revision: format!("sha256:{}", "a".repeat(64)),
            start_ms: 0,
            end_ms: 10_000,
            reference_text: None,
            reference_text_confirmed: false,
        },
    }
}
#[test]
fn model_identity_is_frozen_and_unsupported_controls_are_rejected() {
    let mut value = request();
    assert!(validate_request(&value).is_ok());
    let original = request_digest(&value).unwrap();
    value.model = super::super::VoiceModel::VoxCpm2;
    assert_ne!(original, request_digest(&value).unwrap());
    value.source = VoiceCandidateSource::Designed {
        description: "温暖的女声".into(),
    };
    assert!(validate_request(&value).is_ok());
    value.model = super::super::VoiceModel::MossNano;
    assert!(validate_request(&value).is_err());
}
#[test]
fn lightweight_cloning_does_not_silently_ignore_transcript_conditioning() {
    let mut value = request();
    if let VoiceCandidateSource::Reference {
        reference_text,
        reference_text_confirmed,
        ..
    } = &mut value.source
    {
        *reference_text = Some("参考音频内容".into());
        *reference_text_confirmed = true;
    }
    assert!(validate_request(&value).is_err());
}
