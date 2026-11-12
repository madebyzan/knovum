use super::*;
use crate::local_resources::model_catalog::LocalModelId;

#[test]
fn dubbing_keeps_separations_by_any_model_including_retired_ones_and_rejects_foreign_provenance() {
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
        background_separation: Some(SourceSeparationProvenance {
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
    assert!(
        validate(&config).is_err(),
        "a separation always names its model"
    );
    for model_spec_id in [
        LocalModelId::HtDemucs.separation_spec().unwrap(),
        LocalModelId::HtDemucsF16.separation_spec().unwrap(),
        "spleeter-2stems-fp16-cpu-v1",
    ] {
        config.background_separation.as_mut().unwrap().model_spec_id = model_spec_id.into();
        assert!(validate(&config).is_ok());
    }
    config.background_separation.as_mut().unwrap().model_spec_id =
        LocalModelId::HtDemucsF16.separation_spec().unwrap().into();
    config
        .background_separation
        .as_mut()
        .unwrap()
        .source
        .source_revision = format!("sha256:{}", "c".repeat(64));
    assert!(validate(&config).is_err());
}

#[test]
fn auto_video_background_allows_preserving_original_outside_dialogue_without_manual_separation() {
    let source = SpeechMediaSource {
        asset_path: "/managed/video.mp4".into(),
        source_revision: format!("sha256:{}", "a".repeat(64)),
        duration_ms: 5_000,
    };
    let config = VideoDubbingOutputConfig {
        format: "mp4".into(),
        video: source.clone(),
        background: Some(source),
        background_separation: None,
        voice_gain_db: 0.0,
        background_gain_db: 0.0,
        translation: None,
        preserve_original_outside_dialogue: true,
        source_dialogue_intervals: vec![SourceDialogueInterval {
            start_ms: 500,
            end_ms: 2_000,
        }],
        retain_original_track: true,
    };
    assert!(config.uses_auto_video_background());
    assert!(validate(&config).is_ok());
}
