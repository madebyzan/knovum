use super::{speech_span, FRAME_MS, GUARD_MS};

const SAMPLE_RATE: u32 = 24_000;
/// 10 ms at 24 kHz, the frame speech is looked for in.
const FRAME_SAMPLES: usize = 240;
const GUARD_SAMPLES: usize = 480;

/// `silent` frames of silence, then `speech` frames of speech, then `silent` again.
fn padded(silent: usize, speech: usize) -> Vec<u8> {
    let mut pcm = Vec::new();
    for _ in 0..silent * FRAME_SAMPLES {
        pcm.extend_from_slice(&0.0_f32.to_le_bytes());
    }
    for _ in 0..speech * FRAME_SAMPLES {
        pcm.extend_from_slice(&0.5_f32.to_le_bytes());
    }
    for _ in 0..silent * FRAME_SAMPLES {
        pcm.extend_from_slice(&0.0_f32.to_le_bytes());
    }
    pcm
}

#[test]
fn padding_around_the_speech_is_removed_except_for_a_guard() {
    let pcm = padded(30, 10);
    let span = speech_span(&pcm, SAMPLE_RATE).unwrap();
    assert_eq!(
        span.start,
        (30 * FRAME_SAMPLES - GUARD_SAMPLES) * 4,
        "the guard keeps 20 ms before the first word"
    );
    assert_eq!(
        span.end,
        (40 * FRAME_SAMPLES + GUARD_SAMPLES) * 4,
        "the guard keeps 20 ms after the last word"
    );
    assert_eq!(FRAME_MS * 24, FRAME_SAMPLES as u64);
    assert_eq!(GUARD_MS * 24, GUARD_SAMPLES as u64);
}

#[test]
fn speech_that_reaches_the_edges_is_kept_whole() {
    let pcm = padded(0, 10);
    assert_eq!(speech_span(&pcm, SAMPLE_RATE).unwrap(), 0..pcm.len());
}

#[test]
fn a_room_tone_floor_is_not_mistaken_for_speech() {
    let mut pcm = Vec::new();
    for _ in 0..10 * FRAME_SAMPLES {
        pcm.extend_from_slice(&0.001_f32.to_le_bytes());
    }
    assert!(speech_span(&pcm, SAMPLE_RATE).is_err());
}

#[test]
fn a_line_that_is_silent_throughout_is_invalid_output() {
    let pcm = padded(10, 0);
    assert_eq!(
        speech_span(&pcm, SAMPLE_RATE),
        Err("invalid_output: synthesis returned silence".into())
    );
}
