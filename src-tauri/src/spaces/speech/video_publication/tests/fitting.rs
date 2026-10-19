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

#[test]
fn speech_shorter_than_its_window_keeps_its_own_pace() {
    assert_eq!(
        planned(&[(1.0, 1_000, 3_000)], VIDEO_SAMPLES),
        [(0, 48_000, 48_000)],
        "a line that fits is never slowed down to fill the window it was given"
    );
}

#[test]
fn speech_slightly_longer_than_its_window_is_compressed_into_it() {
    assert_eq!(
        planned(&[(1.1, 1_000, 2_000)], VIDEO_SAMPLES),
        [(0, 48_000, 48_000)],
        "a line within the compression the ear tolerates stays in sync with the picture"
    );
}

#[test]
fn speech_far_longer_than_its_window_speaks_into_the_silence_that_follows() {
    let compressed = (4.0 * f64::from(SAMPLE_RATE) / MAX_COMPRESSION).ceil() as u64;
    assert_eq!(
        planned(&[(4.0, 1_000, 2_000)], VIDEO_SAMPLES),
        [(0, 48_000, compressed)],
        "the line keeps speaking past its window rather than being cut or rejected"
    );
    assert!(compressed > 48_000);
}

#[test]
fn a_line_is_never_heard_past_the_end_of_the_video() {
    assert_eq!(
        planned(&[(4.0, 2_000, 3_000)], 3 * u64::from(SAMPLE_RATE)),
        [(0, 96_000, 48_000)],
        "the last line keeps its tolerable pace and stops with the picture"
    );
}

#[test]
fn a_line_that_overruns_delays_the_next_one_instead_of_talking_over_it() {
    let overrun = (4.0 * f64::from(SAMPLE_RATE) / MAX_COMPRESSION).ceil() as u64;
    let delayed = (1.0 * f64::from(SAMPLE_RATE) / MAX_COMPRESSION).ceil() as u64;
    assert_eq!(
        planned(&[(4.0, 0, 1_000), (1.0, 1_000, 2_000)], VIDEO_SAMPLES),
        [(0, 0, overrun), (1, overrun, delayed)],
        "the next line waits for the floor instead of two voices speaking at once"
    );
}

#[test]
fn a_late_line_compresses_as_far_as_it_may_to_catch_up() {
    let overrun = (4.0 * f64::from(SAMPLE_RATE) / MAX_COMPRESSION).ceil() as u64;
    let caught_up = (2.0 * f64::from(SAMPLE_RATE) / MAX_COMPRESSION).ceil() as u64;
    let last = (1.0 * f64::from(SAMPLE_RATE) / MAX_COMPRESSION).ceil() as u64;
    assert_eq!(
        planned(
            &[(4.0, 0, 1_000), (2.0, 1_000, 3_000), (1.0, 3_000, 4_000)],
            VIDEO_SAMPLES
        ),
        [
            (0, 0, overrun),
            (1, overrun, caught_up),
            (2, overrun + caught_up, last)
        ],
        "a line pushed past its window speeds up so the lateness stops growing"
    );
}

#[test]
fn lateness_is_paid_back_by_the_silence_between_lines() {
    let overrun = (4.0 * f64::from(SAMPLE_RATE) / MAX_COMPRESSION).ceil() as u64;
    assert_eq!(
        planned(&[(4.0, 0, 1_000), (1.0, 10_000, 11_000)], VIDEO_SAMPLES),
        [(0, 0, overrun), (1, 480_000, 48_000)],
        "the next line keeps its own subtitle time once the gap has absorbed the overrun"
    );
}

#[test]
fn a_line_pushed_past_the_picture_is_not_heard() {
    assert_eq!(
        planned(
            &[(4.0, 0, 1_000), (1.0, 1_000, 2_000)],
            3 * u64::from(SAMPLE_RATE)
        ),
        [(0, 0, 3 * u64::from(SAMPLE_RATE))],
        "a line with no picture left to speak over is dropped rather than overlapping"
    );
}

#[test]
fn a_window_that_starts_after_the_video_ends_is_refused() {
    let renders = [render(1.0)];
    let windows = [(Some(10_000), Some(11_000))];
    assert!(timed_renders(&renders, &windows, 3 * u64::from(SAMPLE_RATE)).is_err());
}
