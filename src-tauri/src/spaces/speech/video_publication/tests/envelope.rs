use tokio_util::sync::CancellationToken;

use super::super::{fitting::Placement, SAMPLE_RATE};

const ONE_SECOND: u64 = SAMPLE_RATE as u64;

fn placement(start_ms: u64, duration_ms: u64) -> Placement {
    Placement {
        index: 0,
        start_sample: start_ms * u64::from(SAMPLE_RATE) / 1000,
        sample_count: duration_ms * u64::from(SAMPLE_RATE) / 1000,
        tempo: 1.0,
    }
}

fn read(path: &std::path::Path) -> Vec<f32> {
    hound::WavReader::open(path)
        .unwrap()
        .samples::<f32>()
        .map(|sample| sample.unwrap())
        .collect()
}

struct Workspace(std::path::PathBuf);
impl Workspace {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("knovum-envelope-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn duck_envelope_dips_before_dialogue_and_recovers_slowly_after() {
    let work = Workspace::new();
    let path = work.0.join("duck.wav");
    // One second of picture with dialogue from 400 ms to 600 ms.
    super::write_duck(
        &path,
        ONE_SECOND,
        &[placement(400, 200)],
        &CancellationToken::new(),
    )
    .unwrap();
    let samples = read(&path);
    assert_eq!(samples.len() as u64, ONE_SECOND);
    let ducked = super::super::mastering::duck_level();
    let at = |ms: u64| samples[(ms * u64::from(SAMPLE_RATE) / 1000) as usize];
    assert_eq!(at(0), 1.0, "the bed plays untouched outside dialogue");
    assert_eq!(at(190), 1.0, "the dip has not started before its lookahead");
    assert!(
        (at(400) - ducked).abs() < 0.001,
        "the bed is already down when the first word starts"
    );
    assert!((at(500) - ducked).abs() < 0.001, "plateau under dialogue");
    assert!(
        at(700) > ducked + 0.1 && at(700) < 1.0,
        "the bed recovers gradually across its release tail"
    );
    assert!(
        (at(960) - 1.0).abs() < 0.001,
        "recovery completes by the end of the release"
    );
    assert_eq!(samples[ONE_SECOND as usize - 1], 1.0);
}

#[test]
fn duck_envelope_merges_nearby_lines_into_one_dip() {
    let work = Workspace::new();
    let path = work.0.join("duck.wav");
    super::write_duck(
        &path,
        ONE_SECOND,
        &[placement(100, 100), placement(250, 100)],
        &CancellationToken::new(),
    )
    .unwrap();
    let samples = read(&path);
    let ducked = super::super::mastering::duck_level();
    let at = |ms: u64| samples[(ms * u64::from(SAMPLE_RATE) / 1000) as usize];
    assert!(
        (at(225) - ducked).abs() < 0.001,
        "the gap between two close lines stays ducked instead of pumping"
    );
}

#[test]
fn swap_envelope_selects_bed_inside_windows_and_original_outside() {
    let work = Workspace::new();
    let path = work.0.join("swap.wav");
    super::write_swap(
        &path,
        ONE_SECOND,
        &[super::super::super::video_config::SourceDialogueInterval {
            start_ms: 200,
            end_ms: 800,
        }],
        &[placement(300, 200)],
        &CancellationToken::new(),
    )
    .unwrap();
    let samples = read(&path);
    assert_eq!(samples[0], 0.0, "preserved original outside every window");
    assert!(
        (samples[200 * 48 + 480] - 1.0).abs() < 0.001,
        "the bed is fully selected 10 ms inside the window"
    );
    assert_eq!(samples[500 * 48], 1.0, "merged windows hold the bed");
    assert!(
        (samples[800 * 48 - 481] - 1.0).abs() < 0.001,
        "the bed is held until 10 ms before the window ends"
    );
    assert_eq!(
        samples[ONE_SECOND as usize - 1],
        0.0,
        "the crossfade completes by the window end"
    );
}
