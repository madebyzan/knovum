use super::super::{mixing::timed_renders, SAMPLE_RATE};
use super::{plan, MAX_TEMPO, MIN_TEMPO};
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

#[test]
fn speech_shorter_than_its_window_slows_down_to_match_scene_pacing() {
    assert_eq!(
        planned(&[(1.6, 1_000, 3_000)], VIDEO_SAMPLES),
        [(0, 48_000, 96_000)],
        "a line shorter than its window decelerates to span the speaking window"
    );
}

#[test]
fn speech_within_tolerance_slows_down_up_to_minimum_tempo() {
    let min_rate = (1.0 * f64::from(SAMPLE_RATE) / MIN_TEMPO).round() as u64;
    assert_eq!(
        planned(&[(1.0, 1_000, 4_000)], VIDEO_SAMPLES),
        [(0, 48_000, min_rate)],
        "very short speech decelerates to natural speech bounds without artificial distortion"
    );
}

#[test]
fn speech_slightly_longer_than_its_window_speeds_up_into_it() {
    assert_eq!(
        planned(&[(2.5, 1_000, 3_000)], VIDEO_SAMPLES),
        [(0, 48_000, 96_000)],
        "a line within the compression bounds speeds up to match visual delivery"
    );
}

#[test]
fn speech_far_longer_than_its_window_speaks_into_the_silence_that_follows() {
    let compressed = (4.0 * f64::from(SAMPLE_RATE) / MAX_TEMPO).round() as u64;
    assert_eq!(
        planned(&[(4.0, 1_000, 2_000)], VIDEO_SAMPLES),
        [(0, 48_000, compressed)],
        "the line keeps speaking into following silence rather than cutting off"
    );
    assert!(compressed > 48_000);
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
fn consecutive_lines_stay_anchored_to_their_video_time_without_collision() {
    assert_eq!(
        planned(&[(4.0, 0, 1_000), (1.0, 1_000, 2_000)], VIDEO_SAMPLES),
        [(0, 0, 48_000), (1, 48_000, 48_000)],
        "lines without silence between them fit their exact slots without talking over each other"
    );
}

#[test]
fn lateness_does_not_accumulate_across_the_video_timeline() {
    let compressed = (4.0 * f64::from(SAMPLE_RATE) / MAX_TEMPO).round() as u64;
    assert_eq!(
        planned(&[(4.0, 0, 1_000), (1.0, 10_000, 11_000)], VIDEO_SAMPLES),
        [(0, 0, compressed), (1, 480_000, 48_000)],
        "the next line keeps its exact timestamp and does not suffer cumulative drift"
    );
}

#[test]
fn a_window_that_starts_after_the_video_ends_is_refused() {
    let renders = [render(1.0)];
    let windows = [(Some(10_000), Some(11_000))];
    assert!(timed_renders(&renders, &windows, 3 * u64::from(SAMPLE_RATE)).is_err());
}
