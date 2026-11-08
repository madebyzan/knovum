//! EBU R128 loudness measurement shared by audiobook encoding and the video dubbing mix.
//!
//! FFmpeg's `loudnorm` runs only as an analysis pass. Gains are applied afterwards as static
//! `volume` filters: a static gain is sample-exact, has zero lookahead latency, and never
//! resamples internally, while the dynamic `loudnorm` render filter can shift timing and upsamples
//! to 192 kHz. Audiobook encoding is the one path that keeps the linear `loudnorm` render filter,
//! because its delivery contract is defined in terms of that filter.

use std::{
    cmp::Ordering,
    path::Path,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::{foundation::subprocess, spaces::db};

/// A delivery loudness goal: integrated loudness in LUFS and a true-peak ceiling in dBTP.
#[derive(Clone, Copy, Debug)]
pub(in crate::spaces::speech) struct LoudnessTarget {
    pub loudness_lufs: f64,
    pub true_peak_db: f64,
}

/// The measured program loudness of one audio source, as reported by FFmpeg's `loudnorm`
/// analysis pass.
#[derive(Clone, Deserialize, Serialize)]
pub(in crate::spaces::speech) struct LoudnessMeasurement {
    #[serde(deserialize_with = "deserialize_measured_number")]
    pub(in crate::spaces::speech) input_i: f64,
    #[serde(deserialize_with = "deserialize_measured_number")]
    pub(in crate::spaces::speech) input_tp: f64,
    #[serde(deserialize_with = "deserialize_measured_number")]
    pub(in crate::spaces::speech) input_lra: f64,
    #[serde(deserialize_with = "deserialize_measured_number")]
    pub(in crate::spaces::speech) input_thresh: f64,
    #[serde(deserialize_with = "deserialize_measured_number")]
    pub(in crate::spaces::speech) target_offset: f64,
}

impl LoudnessMeasurement {
    pub(in crate::spaces::speech) fn validate(self) -> Result<Self, String> {
        for (value, minimum, maximum, label) in [
            (self.input_i, -200.0, 50.0, "input_i"),
            (self.input_tp, -200.0, 50.0, "input_tp"),
            (self.input_lra, 0.0, 200.0, "input_lra"),
            (self.input_thresh, -200.0, 50.0, "input_thresh"),
            (self.target_offset, -100.0, 100.0, "target_offset"),
        ] {
            if !value.is_finite() || !(minimum..=maximum).contains(&value) {
                return Err(format!(
                    "FFmpeg loudness measurement {label} is out of range"
                ));
            }
        }
        Ok(self)
    }
}

/// FFmpeg prints `-inf` rather than a number when a source is digital silence. Infinities are
/// admitted here and turned into "leave the source alone" by [`static_gain_db`]; contracts that
/// require an actual program level reject them through [`LoudnessMeasurement::validate`].
fn deserialize_measured_number<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(value) => value
            .as_f64()
            .ok_or_else(|| D::Error::custom("loudness measurement must be numeric")),
        serde_json::Value::String(value) => value
            .parse::<f64>()
            .map_err(|_| D::Error::custom("loudness measurement must be numeric")),
        _ => Err(D::Error::custom("loudness measurement must be numeric")),
    }
}

pub(in crate::spaces::speech) enum LoudnessInput<'a> {
    /// A single media file; its first audio stream is measured.
    File(&'a Path),
    /// An FFmpeg concat demuxer list.
    Concat(&'a Path),
}

/// Below this measured integrated loudness a source counts as digital silence: there is no
/// program loudness to normalize, and any finite gain applied to silence is arbitrary.
const SILENCE_FLOOR_LUFS: f64 = -50.0;

/// Static gains are bounded so a near-silent source is never pushed into being noise and a
/// normalization never needs an extreme boost.
const MAX_STATIC_GAIN_DB: f64 = 24.0;

/// The single static gain that brings `measured` to `target`: loudness-limited and true-peak
/// limited at once, so the normalized source respects both parts of the delivery goal.
pub(in crate::spaces::speech) fn static_gain_db(
    measured: &LoudnessMeasurement,
    target: &LoudnessTarget,
) -> f64 {
    let above_floor = measured
        .input_i
        .partial_cmp(&SILENCE_FLOOR_LUFS)
        .is_some_and(Ordering::is_gt);
    if !above_floor {
        return 0.0;
    }
    let loudness_gain = target.loudness_lufs - measured.input_i;
    let peak_gain = target.true_peak_db - measured.input_tp;
    loudness_gain
        .min(peak_gain)
        .clamp(-MAX_STATIC_GAIN_DB, MAX_STATIC_GAIN_DB)
}

/// Runs the FFmpeg `loudnorm` analysis pass over `input` and returns the measurement it prints.
pub(in crate::spaces::speech) fn measure(
    ffmpeg: &Path,
    input: LoudnessInput<'_>,
    sample_rate: u32,
    target: &LoudnessTarget,
    cancellation: &CancellationToken,
) -> Result<LoudnessMeasurement, String> {
    let filter = format!(
        "[0:a:0]aresample={sample_rate},loudnorm=I={}:TP={}:LRA=11:print_format=json[out]",
        target.loudness_lufs, target.true_peak_db,
    );
    let mut command = Command::new(ffmpeg);
    command.args(["-hide_banner", "-nostdin"]);
    match input {
        LoudnessInput::File(path) => {
            command.arg("-i").arg(path);
        }
        LoudnessInput::Concat(path) => {
            command.args(["-f", "concat", "-safe", "0", "-i"]).arg(path);
        }
    }
    let child = command
        .arg("-filter_complex")
        .arg(filter)
        .args(["-map", "[out]", "-f", "null", "-"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start FFmpeg loudness analysis: {error}"))?;
    let outcome = subprocess::wait_with_stderr(child, cancellation, "cancelled", |_| Ok(()))?;
    if !outcome.status.success() {
        return Err(subprocess::diagnostic_error(
            &outcome.diagnostics,
            "FFmpeg loudness analysis failed",
        ));
    }
    let diagnostics = outcome.diagnostics.join("\n");
    let start = diagnostics
        .rfind('{')
        .ok_or("FFmpeg loudness analysis returned no measurement")?;
    let end = diagnostics[start..]
        .find('}')
        .map(|offset| start + offset + 1)
        .ok_or("FFmpeg loudness measurement is incomplete")?;
    serde_json::from_str::<LoudnessMeasurement>(&diagnostics[start..end]).map_err(db::json_error)
}
