mod lifecycle;

use super::render::ffmpeg_arguments;
use crate::spaces::derivation::ass_format::timestamp as ass_timestamp;

fn cues(value: serde_json::Value) -> Vec<crate::media::transcript::SubtitleCue> {
    serde_json::from_value(value).unwrap()
}

#[test]
fn formats_ass_timestamp() {
    assert_eq!(ass_timestamp(3_723_045), "1:02:03.04");
}

#[test]
fn burns_ass_subtitles_into_h264_video() {
    let arguments =
        ffmpeg_arguments("/source/video.mp4", "subtitle-job.ass").expect("supported platform");
    assert!(arguments
        .iter()
        .any(|value| value == "subtitles=subtitle-job.ass"));
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
    assert!(super::ass::from_cues(&cues(invalid), 2000, None).is_err());
    assert!(super::ass::from_cues(&cues(json!([])), 2000, None).is_err());
    let stale = json!({ "mediaKnowledge": { "schemaVersion": 4, "sourceRevision": "old" } });
    assert!(super::start::automatic_subtitles(&stale, "current").is_err());
}

#[test]
fn out_of_duration_or_unordered_cues_cannot_be_rendered() {
    use serde_json::json;
    assert!(super::ass::from_cues(
        &cues(json!([{ "startMs": 0, "endMs": 3000, "text": "Long" }])),
        2000,
        None
    )
    .is_err());
    assert!(super::ass::from_cues(
        &cues(json!([
            { "startMs": 1000, "endMs": 1500, "text": "Later" },
            { "startMs": 0, "endMs": 500, "text": "Earlier" },
        ])),
        2000,
        None
    )
    .is_err());
}

#[test]
fn splits_long_cjk_and_latin_cues_into_broadcast_ass_events() {
    use serde_json::json;
    let long_latin = "This is a very long spoken sentence that should be split cleanly at natural phrase boundaries for readability.";
    let ass = super::ass::from_cues(
        &cues(json!([{ "startMs": 0, "endMs": 6000, "text": long_latin }])),
        6000,
        Some((1920, 1080)),
    )
    .expect("valid ass");
    assert!(ass.contains("PlayResX: 1280"));
    assert!(ass.contains("PlayResY: 720"));
    assert!(ass.contains("WrapStyle: 0"));
    assert!(ass.contains(crate::desktop::system_font::SYSTEM_SANS_FAMILY));
    let dialogues: Vec<_> = ass.lines().filter(|l| l.starts_with("Dialogue:")).collect();
    assert!(dialogues.len() >= 2, "long Latin cue must split: {ass}");

    let long_cjk = "这是一段非常长的中文口播字幕内容，需要在自然的标点位置拆分成两段甚至三段，保证观众能够轻松阅读。";
    let cjk_ass = super::ass::from_cues(
        &cues(json!([{ "startMs": 0, "endMs": 6000, "text": long_cjk }])),
        6000,
        Some((1080, 1920)),
    )
    .expect("valid vertical cjk ass");
    assert!(cjk_ass.contains("PlayResX: 405"));
    assert!(
        cjk_ass
            .lines()
            .filter(|l| l.starts_with("Dialogue:"))
            .count()
            >= 2
    );
}

#[test]
fn repairs_legacy_glued_latin_subtitle_cues_from_spaced_segments() {
    use serde_json::json;
    let mut video = json!({
        "intrinsicWidth": 1920,
        "intrinsicHeight": 1080,
        "mediaKnowledge": crate::media::transcript::tests::knowledge_fixture(
            "rev-1",
            vec![crate::media::transcript::TranscriptSegment {
                start_ms: 0,
                end_ms: 4000,
                text: "I auditioned when I was nine and started filming".into(),
            }],
        )
    });
    video["mediaKnowledge"]["transcript"]["subtitleCues"] = json!([{
        "startMs": 0,
        "endMs": 4000,
        "text": "IauditionedwhenIwasnineandstartedfilming"
    }]);
    let prepared = super::start::automatic_subtitles(&video, "rev-1").expect("repaired");
    match prepared {
        super::preparation::PreparedSubtitles::Ready(ass) => {
            assert!(
                ass.contains("I auditioned when I was nine"),
                "must recover spaced English words: {ass}"
            );
        }
        super::preparation::PreparedSubtitles::Recognize => panic!("expected ready subtitles"),
    }
}
