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

#[test]
fn preserves_spaces_between_trimmed_latin_words_and_attaches_punctuation() {
    let subtitle_cues = cues([
        token(0, 300, "I"),
        token(300, 600, "auditioned"),
        token(600, 800, "when"),
        token(800, 950, "I"),
        token(950, 1_100, "was"),
        token(1_100, 1_400, "nine"),
        token(1_400, 1_500, ","),
        token(1_500, 1_700, "and"),
        token(1_700, 1_900, "started"),
        token(1_900, 2_200, "filming."),
    ]);
    assert_eq!(subtitle_cues.len(), 1);
    assert_eq!(
        subtitle_cues[0].text,
        "I auditioned when I was nine, and started filming."
    );
}

#[test]
fn inserts_space_at_cjk_latin_boundaries_without_splitting_cjk_runs() {
    let subtitle_cues = cues([
        token(0, 400, "使用"),
        token(400, 800, "Knovum"),
        token(800, 1_200, "剪辑"),
    ]);
    assert_eq!(subtitle_cues.len(), 1);
    assert_eq!(subtitle_cues[0].text, "使用 Knovum 剪辑");
}
