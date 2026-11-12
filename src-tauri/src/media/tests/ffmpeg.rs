use super::stream_codec;

#[test]
fn a_stream_summary_names_its_codec() {
    assert_eq!(
        stream_codec(
            "  Stream #0:0[0x1](und): Video: hevc (Main) (hvc1 / 0x31637668), yuv420p(tv), 1920x1080",
            "Video"
        )
        .as_deref(),
        Some("hevc")
    );
    assert_eq!(
        stream_codec(
            "    Stream #0:1: Video: prores (HQ) (apch / 0x68637061), yuv422p10le",
            "Video"
        )
        .as_deref(),
        Some("prores")
    );
    assert_eq!(
        stream_codec("  Stream #0:0: Video: h264, yuv420p, 1280x720", "Video").as_deref(),
        Some("h264")
    );
    assert!(stream_codec("  Stream #0:1(und): Audio: aac (LC), 48000 Hz", "Video").is_none());
    assert!(stream_codec("Output #0, null, to 'pipe:':", "Video").is_none());
}
