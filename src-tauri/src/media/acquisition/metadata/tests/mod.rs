use super::{MediaKind, YtDlpMetadata};

fn metadata(vcodec: &str, acodec: Option<&str>) -> YtDlpMetadata {
    serde_json::from_value(serde_json::json!({
        "id": "media-1", "title": "Video", "extractor_key": "Youtube",
        "webpage_url": "https://www.youtube.com/watch?v=media-1",
        "filepath": "/assets/media.mp4", "vcodec": vcodec, "acodec": acodec,
    }))
    .unwrap()
}

#[test]
fn mp4_container_does_not_make_av1_playable() {
    assert!(!metadata("av01.0.08M.08", Some("mp4a.40.2")).is_playable(MediaKind::Video, "mp4"));
    assert!(metadata("avc1.640028", Some("mp4a.40.2")).is_playable(MediaKind::Video, "mp4"));
}

#[test]
fn an_audio_asset_is_playable_only_in_an_aac_container() {
    assert!(!metadata("none", Some("opus")).is_playable(MediaKind::Audio, "webm"));
    assert!(metadata("none", Some("mp4a.40.2")).is_playable(MediaKind::Audio, "m4a"));
}

#[test]
fn missing_acodec_is_not_playable_and_triggers_transcode() {
    assert!(!metadata("avc1.640028", None).is_playable(MediaKind::Video, "mp4"));
    assert!(!metadata("none", None).is_playable(MediaKind::Audio, "m4a"));
}
