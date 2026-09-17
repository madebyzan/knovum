use super::{cues, TimedToken};

fn token(start_ms: u64, end_ms: u64, text: &str) -> TimedToken {
    TimedToken {
        start_ms,
        end_ms,
        text: text.into(),
    }
}

#[test]
fn uses_speech_timing_and_semantic_boundaries() {
    let subtitle_cues = cues([
        token(1_990, 2_420, "别"),
        token(2_420, 2_900, "这样说。"),
        token(4_000, 4_500, "更自然"),
    ]);
    assert_eq!(subtitle_cues.len(), 2);
    assert_eq!(
        (subtitle_cues[0].start_ms, subtitle_cues[0].end_ms),
        (1_990, 2_900)
    );
    assert_eq!(subtitle_cues[0].text, "别这样说。");
    assert_eq!(
        (subtitle_cues[1].start_ms, subtitle_cues[1].end_ms),
        (4_000, 4_500)
    );
}

#[test]
fn splits_long_speech_by_elapsed_audio_time() {
    let subtitle_cues = cues([
        token(0, 2_000, "one "),
        token(2_000, 4_000, "two "),
        token(4_000, 5_500, "three"),
    ]);
    assert_eq!(subtitle_cues.len(), 2);
    assert_eq!(subtitle_cues[0].text, "one two");
    assert_eq!(subtitle_cues[1].text, "three");
}
