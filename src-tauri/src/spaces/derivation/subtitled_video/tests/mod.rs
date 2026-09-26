use super::{render::ffmpeg_arguments, srt::timestamp};

#[test]
fn formats_subrip_timestamp() {
    assert_eq!(timestamp(3_723_045), "01:02:03,045");
}

#[test]
fn burns_subtitles_into_h264_video() {
    let arguments =
        ffmpeg_arguments("/source/video.mp4", "subtitle-job.srt").expect("supported platform");
    assert!(arguments
        .iter()
        .any(|value| value.starts_with("subtitles=subtitle-job.srt:")));
    assert!(arguments
        .iter()
        .any(|value| value.contains(crate::system_font::SYSTEM_SANS_FAMILY)));
    assert!(arguments.iter().any(|value| value.contains("WrapStyle=0")));
    let expected = if cfg!(target_os = "macos") {
        "h264_videotoolbox"
    } else {
        "h264_mf"
    };
    assert!(arguments.windows(2).any(|pair| pair == ["-c:v", expected]));
    assert!(!arguments.iter().any(|value| value == "mov_text"));
}
