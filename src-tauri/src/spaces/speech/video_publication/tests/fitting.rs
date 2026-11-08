use super::super::{mixing::timed_renders, SAMPLE_RATE};
use super::{plan, MAX_COMPRESSION};
use crate::spaces::speech::contracts::SpeechSegmentRenderOutput;

/// Two minutes of video, which every window in these tests fits inside.
const VIDEO_SAMPLES: u64 = 120 * 48_000;

fn render(seconds: f64) -> SpeechSegmentRenderOutput {
    SpeechSegmentRenderOutput {
        segment_id: "segment".into(),
        path: "/assets/segment.wav".into(),
        sha256: "digest".into(),
        sample_rate: SAMPLE_RATE,
        sample_count: (seconds * f64::from(SAMPLE_RATE)) as u64,
        cached: false,
    }
}

/// Where every line of `(spoken seconds, window start ms, window end ms)` ends up being heard,
/// as `(line, start sample, sample count)`.
fn planned(lines: &[(f64, i64, i64)], video_samples: u64) -> Vec<(usize, u64, u64)> {
    let renders = lines
        .iter()
        .map(|(seconds, _, _)| render(*seconds))
        .collect::<Vec<_>>();
    let windows = lines
        .iter()
        .map(|(_, start, end)| (Some(*start), Some(*end)))
        .collect::<Vec<_>>();
    let timed = timed_renders(&renders, &windows, video_samples).unwrap();
    plan(&timed, video_samples)
        .unwrap()
        .iter()
        .map(|placement| {
            (
                placement.index,
                placement.start_sample,
                placement.sample_count,
            )
        })
        .collect()
}

fn planned_tempos(lines: &[(f64, i64, i64)], video_samples: u64) -> Vec<f64> {
    let renders = lines
        .iter()
        .map(|(seconds, _, _)| render(*seconds))
        .collect::<Vec<_>>();
    let windows = lines
        .iter()
        .map(|(_, start, end)| (Some(*start), Some(*end)))
        .collect::<Vec<_>>();
    let timed = timed_renders(&renders, &windows, video_samples).unwrap();
    plan(&timed, video_samples)
        .unwrap()
        .iter()
        .map(|placement| placement.tempo)
        .collect()
}

#[test]
fn speech_shorter_than_its_window_plays_at_pristine_rate_without_slowing_down() {
    // 1.6s speech in a 3.0s window: plays exactly for 1.6s (76,800 samples) at 1.0x.
    // NEVER slowed down to fill the window; no atempo filter, zero phase/metallic rasping.
    assert_eq!(
        planned(&[(1.6, 1_000, 4_000)], VIDEO_SAMPLES),
        [(0, 48_000, 76_800)],
        "shorter speech retains pristine 1.0x rate without artificial deceleration"
    );
    assert_eq!(
        planned_tempos(&[(1.6, 1_000, 4_000)], VIDEO_SAMPLES),
        [1.0],
        "tempo must be strictly 1.0"
    );
}

#[test]
fn speech_extending_past_window_into_following_silence_plays_at_pristine_rate() {
    // 2.5s speech in a 2.0s window [1s, 3s], but next line is at 5s.
    // Because 2.5s easily fits before 5s without colliding, it plays at pristine 1.0x rate.
    assert_eq!(
        planned(&[(2.5, 1_000, 3_000), (1.0, 5_000, 6_000)], VIDEO_SAMPLES),
        [(0, 48_000, 120_000), (1, 240_000, 48_000)],
        "speech comfortably uses inter-dialogue silence without artificial speedup"
    );
    assert_eq!(
        planned_tempos(&[(2.5, 1_000, 3_000), (1.0, 5_000, 6_000)], VIDEO_SAMPLES),
        [1.0, 1.0],
        "both lines retain 1.0x uncompressed rate"
    );
}

#[test]
fn speech_colliding_with_next_line_compresses_cleanly_up_to_max_compression() {
    // 2.4s speech starting at 1.0s, with next line starting at 3.0s (room is 2.0s).
    // It compresses by 2.4 / 2.0 = 1.20x to fit in the 2.0s room (96,000 samples).
    assert_eq!(
        planned(&[(2.4, 1_000, 3_000), (1.0, 3_000, 4_000)], VIDEO_SAMPLES),
        [(0, 48_000, 96_000), (1, 144_000, 48_000)],
        "compresses cleanly to prevent speaking over the next speaker"
    );
    let tempos = planned_tempos(&[(2.4, 1_000, 3_000), (1.0, 3_000, 4_000)], VIDEO_SAMPLES);
    assert!((tempos[0] - 1.20).abs() < 0.001);
    assert_eq!(tempos[1], 1.0);
}

#[test]
fn speech_slightly_over_its_room_is_rescued_by_compression_instead_of_truncation() {
    // 2.7s speech with 2.0s of room before the next speaker: 1.35x compression speaks the whole
    // line (96,000 samples) with nothing cut, where a 1.25x cap would truncate ~0.16s off the end.
    assert_eq!(
        planned(&[(2.7, 1_000, 3_000), (1.0, 3_000, 4_000)], VIDEO_SAMPLES),
        [(0, 48_000, 96_000), (1, 144_000, 48_000)],
        "the complete line is spoken faster instead of being cut"
    );
    let tempos = planned_tempos(&[(2.7, 1_000, 3_000), (1.0, 3_000, 4_000)], VIDEO_SAMPLES);
    assert!((tempos[0] - 1.35).abs() < 0.001);
    assert_eq!(tempos[1], 1.0);
}

#[test]
fn speech_far_longer_than_gap_caps_compression_and_protects_next_speaker() {
    // 4.0s speech starting at 1.0s, next line starts at 2.0s (room is 1.0s).
    // Maximum compression is 1.4x (speech becomes ~2.86s, capped to 1.0s room = 48,000 samples).
    // Next line STILL starts exactly at 2.0s (96,000 samples) with zero delay!
    assert_eq!(
        planned(&[(4.0, 1_000, 2_000), (1.0, 2_000, 3_000)], VIDEO_SAMPLES),
        [(0, 48_000, 48_000), (1, 96_000, 48_000)],
        "caps compression at MAX_COMPRESSION and ensures the next speaker starts on time"
    );
    let tempos = planned_tempos(&[(4.0, 1_000, 2_000), (1.0, 2_000, 3_000)], VIDEO_SAMPLES);
    assert_eq!(tempos[0], MAX_COMPRESSION);
    assert_eq!(tempos[1], 1.0);
}

#[test]
fn a_line_is_never_heard_past_the_end_of_the_video() {
    assert_eq!(
        planned(&[(4.0, 2_000, 3_000)], 3 * u64::from(SAMPLE_RATE)),
        [(0, 96_000, 48_000)],
        "the last line stops with the picture"
    );
}

#[test]
fn consecutive_lines_stay_anchored_to_their_video_time_with_zero_drift() {
    // Three consecutive lines across a timeline:
    // Even if line 0 is long, line 1 starts exactly at 1.0s, and line 2 starts exactly at 2.0s.
    assert_eq!(
        planned(
            &[(2.0, 0, 1_000), (1.5, 1_000, 2_000), (1.0, 2_000, 3_000)],
            VIDEO_SAMPLES
        ),
        [(0, 0, 48_000), (1, 48_000, 48_000), (2, 96_000, 48_000)],
        "each line anchors strictly to its visual start timestamp without cumulative drift"
    );
}
