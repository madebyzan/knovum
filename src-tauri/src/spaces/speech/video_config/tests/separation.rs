use super::*;
use crate::local_resources::model_catalog::LocalModelId;

#[test]
fn dubbing_keeps_both_separation_models_and_rejects_foreign_provenance() {
    let source = SpeechMediaSource {
        asset_path: "/managed/video.mp4".into(),
        source_revision: format!("sha256:{}", "a".repeat(64)),
        duration_ms: 1_000,
    };
    let mut config = VideoDubbingOutputConfig {
        format: "mp4".into(),
        video: source.clone(),
        background: Some(SpeechMediaSource {
            asset_path: "/managed/background.wav".into(),
            source_revision: format!("sha256:{}", "b".repeat(64)),
            duration_ms: 1_000,
        }),
        background_separation: Some(AcceptedSourceSeparation {
            separation_id: uuid::Uuid::new_v4().to_string(),
            model_spec_id: String::new(),
            source,
        }),
        voice_gain_db: 0.0,
        background_gain_db: 0.0,
        translation: None,
        preserve_original_outside_dialogue: false,
        source_dialogue_intervals: Vec::new(),
        retain_original_track: true,
    };
    for model in [
        LocalModelId::Spleeter2Stems,
        LocalModelId::Spleeter2StemsFull,
    ] {
        config.background_separation.as_mut().unwrap().model_spec_id =
            model.separation_spec().unwrap().into();
        assert!(validate(&config).is_ok());
    }
    config.background_separation.as_mut().unwrap().model_spec_id = "unknown-model".into();
    assert!(validate(&config).is_err());
    config.background_separation.as_mut().unwrap().model_spec_id = LocalModelId::Spleeter2StemsFull
        .separation_spec()
        .unwrap()
        .into();
    config
        .background_separation
        .as_mut()
        .unwrap()
        .source
        .source_revision = format!("sha256:{}", "c".repeat(64));
    assert!(validate(&config).is_err());
}
