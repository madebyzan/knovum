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
            design_prompt: None,
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

#[test]
fn reference_duration_bounds_follow_the_selected_synthesis_model() {
    let mut value = request();
    if let VoiceCandidateSource::Reference { end_ms, .. } = &mut value.source {
        *end_ms = 20_000;
    }
    assert!(validate_request(&value).is_ok());
    value.model = super::super::VoiceModel::VoxCpm2;
    assert!(validate_request(&value).is_err());
    if let VoiceCandidateSource::Reference { end_ms, .. } = &mut value.source {
        *end_ms = 15_000;
    }
    assert!(validate_request(&value).is_ok());
}

#[test]
fn reference_cloning_supports_optional_design_prompt_on_capable_models() {
    let mut value = request();
    value.model = super::super::VoiceModel::VoxCpm2;
    if let VoiceCandidateSource::Reference {
        design_prompt,
        end_ms,
        ..
    } = &mut value.source
    {
        *end_ms = 10_000;
        *design_prompt = Some("沉稳大气的男声".into());
    }
    assert!(validate_request(&value).is_ok());

    if let VoiceCandidateSource::Reference { design_prompt, .. } = &mut value.source {
        *design_prompt = Some("包含(括号)的提示词".into());
    }
    assert!(validate_request(&value).is_err());

    if let VoiceCandidateSource::Reference { design_prompt, .. } = &mut value.source {
        *design_prompt = Some("沉稳男声".into());
    }
    value.model = super::super::VoiceModel::MossNano;
    assert!(validate_request(&value).is_err());
}
