use super::super::{
    contracts::SpeechSegmentRenderOutput,
    publication::hash_file,
    video_config::{SourceDialogueInterval, SpeechMediaSource, VideoDubbingOutputConfig},
};
use super::{final_mix::MixOutput, mixing, video_mux, SAMPLE_RATE};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tokio_util::sync::CancellationToken;

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("knovum-audio-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn child(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn wave(path: &Path, seconds: u32, amplitudes: &[f32]) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: amplitudes.len() as u16,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for _ in 0..seconds * SAMPLE_RATE {
        for sample in amplitudes {
            writer.write_sample(*sample).unwrap();
        }
    }
    writer.finalize().unwrap();
}
fn sine(path: &Path, seconds: u32, frequency: f32, amplitude: f32, channels: u16) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for frame in 0..seconds * SAMPLE_RATE {
        let value = amplitude
            * (2.0 * std::f32::consts::PI * frequency * frame as f32 / SAMPLE_RATE as f32).sin();
        for _ in 0..channels {
            writer.write_sample(value).unwrap();
        }
    }
    writer.finalize().unwrap();
}
fn ffmpeg() -> PathBuf {
    std::env::var_os("KNOVUM_FFMPEG_PATH")
        .expect("Set KNOVUM_FFMPEG_PATH for audio integration tests")
        .into()
}
fn source(path: &Path) -> SpeechMediaSource {
    SpeechMediaSource {
        asset_path: path.to_string_lossy().into_owned(),
        source_revision: format!("sha256:{}", hash_file(path).unwrap()),
        duration_ms: 2000,
    }
}
/// The linear level a stem bus carries: the static normalization its element reference demands.
fn reference_linear(
    tool: &Path,
    path: &Path,
    reference: &crate::spaces::speech::loudness::LoudnessTarget,
    cancel: &CancellationToken,
) -> f32 {
    let measured = crate::spaces::speech::loudness::measure(
        tool,
        crate::spaces::speech::loudness::LoudnessInput::File(path),
        SAMPLE_RATE,
        reference,
        cancel,
    )
    .unwrap();
    10f32.powf(crate::spaces::speech::loudness::static_gain_db(&measured, reference) as f32 / 20.0)
}

#[test]
#[ignore = "Requires the explicit FFmpeg test binary"]
fn stems_preserve_stereo_original_outside_source_dialogue_and_timed_voice() {
    let work = Workspace::new();
    let original = work.child("original.wav");
    let background = work.child("background.wav");
    let voice = work.child("voice.wav");
    wave(&original, 2, &[0.2, 0.4]);
    wave(&background, 2, &[0.03, 0.07]);
    wave(&voice, 1, &[0.1]);
    let config = VideoDubbingOutputConfig {
        format: "mp4".into(),
        video: source(&original),
        background: Some(source(&background)),
        background_separation: None,
        voice_gain_db: 0.0,
        background_gain_db: 0.0,
        translation: None,
        preserve_original_outside_dialogue: true,
        source_dialogue_intervals: vec![SourceDialogueInterval {
            start_ms: 500,
            end_ms: 1500,
        }],
        retain_original_track: true,
    };
    let cancel = CancellationToken::new();
    let tool = ffmpeg();
    let bed_work = work.child("bed");
    fs::create_dir(&bed_work).unwrap();
    let bed = mixing::build_stem(
        mixing::MixRequest {
            ffmpeg: &tool,
            work: &bed_work,
            timed: &[],
            video_samples: 96000,
            background: Some(&background),
            config: &config,
            cancellation: &cancel,
        },
        MixOutput::Background,
    )
    .unwrap();
    let mut reader = hound::WavReader::open(bed).unwrap();
    assert_eq!(reader.spec().channels, 2);
    assert_eq!(reader.duration(), 96000);
    let samples = reader
        .samples::<f32>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // The stem carries every bus at its element reference: the preserved original and the
    // separated bed are both normalized to the background reference, so the swap between them
    // is loudness-neutral.
    let original_level = reference_linear(
        &tool,
        &original,
        &super::mastering::BACKGROUND_REFERENCE,
        &cancel,
    );
    let bed_level = reference_linear(
        &tool,
        &background,
        &super::mastering::BACKGROUND_REFERENCE,
        &cancel,
    );
    for (frame, values, level) in [
        (12000, [0.2, 0.4], original_level),
        (48000, [0.03, 0.07], bed_level),
        (84000, [0.2, 0.4], original_level),
    ] {
        assert!((samples[frame * 2] - values[0] * level).abs() < 0.00001);
        assert!((samples[frame * 2 + 1] - values[1] * level).abs() < 0.00001);
    }
    let render = SpeechSegmentRenderOutput {
        segment_id: "segment".into(),
        path: voice.to_string_lossy().into_owned(),
        sha256: hash_file(&voice).unwrap(),
        sample_rate: SAMPLE_RATE,
        sample_count: 48000,
        cached: false,
    };
    let timed = mixing::timed_renders(&[render], &[(Some(500), Some(1500))], 96000).unwrap();
    let voice_work = work.child("dialogue");
    fs::create_dir(&voice_work).unwrap();
    let dialogue = mixing::build_stem(
        mixing::MixRequest {
            ffmpeg: &tool,
            work: &voice_work,
            timed: &timed,
            video_samples: 96000,
            background: Some(&background),
            config: &config,
            cancellation: &cancel,
        },
        MixOutput::Dialogue,
    )
    .unwrap();
    let mut reader = hound::WavReader::open(dialogue).unwrap();
    assert_eq!(reader.spec().channels, 1);
    assert_eq!(reader.duration(), 96000);
    let samples = reader
        .samples::<f32>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let dialogue_level = reference_linear(
        &tool,
        &voice_work.join("dialogue.wav"),
        &super::mastering::DIALOGUE_REFERENCE,
        &cancel,
    );
    assert_eq!(samples[12000], 0.0);
    assert!((samples[48000] - 0.1 * dialogue_level).abs() < 0.00001);
    assert_eq!(samples[84000], 0.0);
}

#[test]
#[ignore = "Requires the explicit FFmpeg test binary"]
fn mux_retains_original_track_without_shortening_video() {
    let work = Workspace::new();
    let tool = ffmpeg();
    let video = work.child("source.mp4");
    assert!(Command::new(&tool)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=size=32x32:rate=10:duration=2",
            "-f",
            "lavfi",
            "-i",
            "sine=duration=1",
            "-c:v",
            "mpeg4",
            "-c:a",
            "aac"
        ])
        .arg(&video)
        .status()
        .unwrap()
        .success());
    let voice = work.child("voice.wav");
    wave(&voice, 2, &[0.1]);
    let output = work.child("dub.mp4");
    video_mux::mux_video(
        &tool,
        &video,
        &voice,
        &output,
        &CancellationToken::new(),
        true,
        2 * u64::from(SAMPLE_RATE),
    )
    .unwrap();
    let duration =
        crate::media::ffmpeg::duration_us(&tool, &output, "0:v:0", &CancellationToken::new())
            .unwrap();
    assert!(duration >= 1_900_000);
    assert!(
        crate::media::ffmpeg::duration_us(&tool, &output, "0:a:1", &CancellationToken::new())
            .unwrap()
            >= 1_900_000
    );
}

#[test]
#[ignore = "Requires the explicit FFmpeg test binary"]
fn combined_mix_is_mastered_to_the_program_reference() {
    let work = Workspace::new();
    let tool = ffmpeg();
    let picture = work.child("picture.wav");
    let bed = work.child("bed.wav");
    let voice = work.child("voice.wav");
    sine(&picture, 2, 440.0, 0.2, 2);
    sine(&bed, 2, 440.0, 0.3, 2);
    sine(&voice, 1, 1_000.0, 0.5, 1);
    let config = VideoDubbingOutputConfig {
        format: "mp4".into(),
        video: source(&picture),
        background: Some(source(&bed)),
        background_separation: None,
        voice_gain_db: 0.0,
        background_gain_db: 0.0,
        translation: None,
        preserve_original_outside_dialogue: false,
        source_dialogue_intervals: vec![],
        retain_original_track: false,
    };
    let render = SpeechSegmentRenderOutput {
        segment_id: "segment".into(),
        path: voice.to_string_lossy().into_owned(),
        sha256: hash_file(&voice).unwrap(),
        sample_rate: SAMPLE_RATE,
        sample_count: 48_000,
        cached: false,
    };
    let timed = mixing::timed_renders(&[render], &[(Some(500), Some(1_500))], 96_000).unwrap();
    let mix_work = work.child("mix");
    fs::create_dir(&mix_work).unwrap();
    let mixed = mixing::build_mix(
        &tool,
        &mix_work,
        &timed,
        96_000,
        Some(&bed),
        &config,
        &CancellationToken::new(),
    )
    .unwrap();
    let measured = crate::spaces::speech::loudness::measure(
        &tool,
        crate::spaces::speech::loudness::LoudnessInput::File(&mixed),
        SAMPLE_RATE,
        &super::mastering::PROGRAM_MASTER,
        &CancellationToken::new(),
    )
    .unwrap();
    assert!(
        (measured.input_i - super::mastering::PROGRAM_MASTER.loudness_lufs).abs() < 1.5,
        "program loudness {} LUFS should sit at the delivery reference",
        measured.input_i
    );
    assert!(
        measured.input_tp <= super::mastering::PROGRAM_MASTER.true_peak_db + 0.5,
        "true peak {} dBTP exceeds the delivery ceiling",
        measured.input_tp
    );
}
