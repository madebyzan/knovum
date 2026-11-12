use super::super::{mixing::timed_renders, SAMPLE_RATE};
use super::{plan, Placement, MAX_COMPRESSION};
use crate::spaces::speech::contracts::SpeechSegmentRenderOutput;

/// Two minutes of video, which every window in these tests fits inside.
const VIDEO_SAMPLES: u64 = 120 * 48_000;
const SECOND: u64 = 48_000;

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

/// The plan for lines of `(spoken seconds, window start ms, window end ms)`.
fn planned(lines: &[(f64, i64, i64)], video_samples: u64) -> Result<Vec<Placement>, String> {
    let renders = lines
        .iter()
        .map(|(seconds, _, _)| render(*seconds))
        .collect::<Vec<_>>();
    let windows = lines
        .iter()
        .map(|(_, start, end)| (Some(*start), Some(*end)))
        .collect::<Vec<_>>();
    plan(
        &timed_renders(&renders, &windows, video_samples).unwrap(),
        video_samples,
    )
}

/// Where every line is heard, as `(line, start sample, sample count)`.
fn placed(lines: &[(f64, i64, i64)]) -> Vec<(usize, u64, u64)> {
    planned(lines, VIDEO_SAMPLES)
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

fn tempos(lines: &[(f64, i64, i64)]) -> Vec<f64> {
    planned(lines, VIDEO_SAMPLES)
        .unwrap()
        .iter()
        .map(|placement| placement.tempo)
        .collect()
}

#[test]
fn a_line_that_fits_its_interval_is_heard_at_its_own_pace() {
    assert_eq!(placed(&[(0.8, 1_000, 2_000)]), [(0, SECOND, 38_400)]);
    assert_eq!(tempos(&[(0.8, 1_000, 2_000)]), [1.0]);
}

#[test]
fn a_line_is_never_slowed_down_to_fill_its_interval() {
    assert_eq!(tempos(&[(0.5, 0, 3_000)]), [1.0]);
}

#[test]
fn a_slightly_long_line_is_compressed_to_end_with_its_interval() {
    let tempo = tempos(&[(1.2, 0, 1_000), (1.0, 5_000, 6_000)])[0];
    assert!((tempo - 1.2).abs() < 1e-9);
    assert_eq!(
        placed(&[(1.2, 0, 1_000), (1.0, 5_000, 6_000)])[0],
        (0, 0, SECOND)
    );
}

#[test]
fn a_line_too_long_for_its_interval_speaks_into_the_silence_after_it_at_its_own_pace() {
    // Two seconds of speech in a one-second interval, with two seconds before the next line.
    assert_eq!(
        placed(&[(2.0, 0, 1_000), (1.0, 3_000, 4_000)]),
        [(0, 0, 2 * SECOND), (1, 3 * SECOND, SECOND)]
    );
    assert_eq!(tempos(&[(2.0, 0, 1_000), (1.0, 3_000, 4_000)]), [1.0, 1.0]);
}

#[test]
fn a_line_compressed_into_the_silence_before_the_next_keeps_the_next_on_time() {
    // 2.4 s of speech with 2 s before the next line: 1.2× ends it exactly on the next onset.
    let placements = planned(&[(2.4, 0, 1_000), (1.0, 2_000, 3_000)], VIDEO_SAMPLES).unwrap();
    assert!((placements[0].tempo - 1.2).abs() < 1e-9);
    assert_eq!(placements[0].end_sample(), 2 * SECOND);
    assert_eq!(placements[1].start_sample, 2 * SECOND);
}

#[test]
fn a_line_that_cannot_fit_is_heard_whole_and_delays_the_next_line() {
    // Three seconds of speech with one second before the next line.
    let placements = planned(
        &[(3.0, 0, 1_000), (0.5, 1_000, 1_500), (0.5, 10_000, 10_500)],
        VIDEO_SAMPLES,
    )
    .unwrap();
    assert_eq!(placements[0].tempo, MAX_COMPRESSION);
    assert_eq!(placements[0].sample_count, 115_200, "never cut");
    assert_eq!(placements[1].start_sample, placements[0].end_sample());
    // The delay is absorbed by the silence before the third line, which starts on time.
    assert_eq!(placements[2].start_sample, 10 * SECOND);
}

#[test]
fn no_two_lines_are_ever_heard_at_once() {
    let placements = planned(
        &[(1.5, 0, 1_000), (1.5, 500, 1_500), (1.5, 1_000, 2_000)],
        VIDEO_SAMPLES,
    )
    .unwrap();
    for pair in placements.windows(2) {
        assert!(pair[0].end_sample() <= pair[1].start_sample);
    }
}

#[test]
fn dialogue_that_cannot_end_with_the_picture_is_refused_rather_than_cut() {
    let error = planned(&[(4.0, 0, 1_000)], 2 * SECOND).err().unwrap();
    assert!(error.contains("past the end of the video"), "{error}");
}

#[test]
fn the_last_line_is_compressed_to_end_with_the_picture() {
    let placements = planned(&[(2.2, 0, 1_000)], 2 * SECOND).unwrap();
    assert!((placements[0].tempo - 1.1).abs() < 1e-9);
    assert_eq!(placements[0].end_sample(), 2 * SECOND);
}
