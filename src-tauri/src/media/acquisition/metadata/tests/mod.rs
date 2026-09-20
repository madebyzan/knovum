    use super::{MediaKind, YtDlpMetadata};

    fn metadata(vcodec: &str, acodec: &str) -> YtDlpMetadata {
        serde_json::from_value(serde_json::json!({
            "id": "media-1",
            "title": "Video",
            "extractor_key": "Youtube",
            "webpage_url": "https://www.youtube.com/watch?v=media-1",
            "filepath": "/assets/media.mp4",
            "vcodec": vcodec,
            "acodec": acodec,
        }))
        .unwrap()
    }

    #[test]
    fn mp4_container_does_not_make_av1_playable() {
        let source = metadata("av01.0.08M.08", "mp4a.40.2");
        assert_eq!(source.media_kind().unwrap(), MediaKind::Video);
        assert!(!source.is_playable(MediaKind::Video, "mp4").unwrap());
        assert!(metadata("avc1.640028", "mp4a.40.2")
            .is_playable(MediaKind::Video, "mp4")
            .unwrap());
    }

    #[test]
    fn audio_kind_uses_the_selected_codec() {
        let source = metadata("none", "opus");
        assert_eq!(source.media_kind().unwrap(), MediaKind::Audio);
        assert!(!source.is_playable(MediaKind::Audio, "webm").unwrap());
    }
