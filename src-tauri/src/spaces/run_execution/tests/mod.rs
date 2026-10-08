use super::*;

#[test]
fn finite_route_assigns_each_owned_step_kind() {
    assert_eq!(
        route("captured_action", &serde_json::json!({})),
        Route::Captured
    );
    assert_eq!(route("action", &serde_json::json!({})), Route::Action);
    assert_eq!(
        route("speech_voice_candidate", &serde_json::json!({})),
        Route::VoiceCandidate
    );
    assert_eq!(
        route("speech_source_separation", &serde_json::json!({})),
        Route::SourceSeparation
    );
    assert_eq!(
        route("speech_video_translation_segment", &serde_json::json!({})),
        Route::Translation
    );
    assert_eq!(route("unknown", &serde_json::json!({})), Route::Unsupported);
    assert_eq!(
        route("action", &serde_json::json!({ "unexpected": true })),
        Route::Unsupported
    );
}
