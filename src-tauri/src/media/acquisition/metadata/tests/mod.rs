use super::{MediaKind, YtDlpMetadata};

fn metadata(vcodec: &str, acodec: &str) -> YtDlpMetadata {
    serde_json::from_value(serde_json::json!({
        "id": "media-1", "title": "Video", "extractor_key": "Youtube",
        "webpage_url": "https://www.youtube.com/watch?v=media-1",
        "filepath": "/assets/media.mp4", "vcodec": vcodec, "acodec": acodec,
    }))
    .unwrap()
}

#[test]
fn mp4_container_does_not_make_av1_playable() {
    assert!(!metadata("av01.0.08M.08", "mp4a.40.2")
        .is_playable(MediaKind::Video, "mp4")
        .unwrap());
    assert!(metadata("avc1.640028", "mp4a.40.2")
        .is_playable(MediaKind::Video, "mp4")
        .unwrap());
}

#[test]
fn an_audio_asset_is_playable_only_in_an_aac_container() {
    assert!(!metadata("none", "opus")
        .is_playable(MediaKind::Audio, "webm")
        .unwrap());
    assert!(metadata("none", "mp4a.40.2")
        .is_playable(MediaKind::Audio, "m4a")
        .unwrap());
}
