mod lifecycle;

use super::{render::ffmpeg_arguments, srt::timestamp};

fn cues(value: serde_json::Value) -> Vec<crate::media::transcript::SubtitleCue> {
    serde_json::from_value(value).unwrap()
}

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

#[test]
fn invalid_or_stale_cues_are_rejected_instead_of_silently_recognized_again() {
    use serde_json::json;
    let invalid = json!([{ "startMs": 1000, "endMs": 500, "text": "Hello" }]);
    assert!(super::srt::from_cues(&cues(invalid), 2000).is_err());
    assert!(super::srt::from_cues(&cues(json!([])), 2000).is_err());
    let stale = json!({ "mediaKnowledge": { "schemaVersion": 4, "sourceRevision": "old" } });
    assert!(super::start::automatic_subtitles(&stale, "current").is_err());
}

#[test]
fn out_of_duration_or_unordered_cues_cannot_be_rendered() {
    use serde_json::json;
    assert!(super::srt::from_cues(
        &cues(json!([{ "startMs": 0, "endMs": 3000, "text": "Long" }])),
        2000
    )
    .is_err());
    assert!(super::srt::from_cues(
        &cues(json!([
            { "startMs": 1000, "endMs": 1500, "text": "Later" },
            { "startMs": 0, "endMs": 500, "text": "Earlier" },
        ])),
        2000
    )
    .is_err());
}
