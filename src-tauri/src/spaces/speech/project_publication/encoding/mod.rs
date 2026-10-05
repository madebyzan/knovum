pub(super) use metadata::write_chapter_metadata;
mod metadata;
pub(super) use sources::{
    normalize_sources, wav_sample_count, write_concat_file, NormalizedSource,
};
mod sources;
use std::{
    path::Path,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::super::project_config::AudiobookOutputConfig;
use crate::{media::subprocess, spaces::db};

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct LoudnessMeasurement {
    #[serde(deserialize_with = "deserialize_finite_number")]
    input_i: f64,
    #[serde(deserialize_with = "deserialize_finite_number")]
    input_tp: f64,
    #[serde(deserialize_with = "deserialize_finite_number")]
    input_lra: f64,
    #[serde(deserialize_with = "deserialize_finite_number")]
    input_thresh: f64,
    #[serde(deserialize_with = "deserialize_finite_number")]
    target_offset: f64,
}

impl LoudnessMeasurement {
    pub(super) fn validate(self) -> Result<Self, String> {
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

fn deserialize_finite_number<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    let number = match value {
        serde_json::Value::Number(value) => value.as_f64(),
        serde_json::Value::String(value) => value.parse::<f64>().ok(),
        _ => None,
    }
    .ok_or_else(|| D::Error::custom("loudness measurement must be numeric"))?;
    if !number.is_finite() {
        return Err(D::Error::custom("loudness measurement must be finite"));
    }
    Ok(number)
}

pub(super) struct EncodeRequest<'a> {
    pub ffmpeg: &'a Path,
    pub concat_path: &'a Path,
    pub output: &'a Path,
    pub format: &'a str,
    pub sample_rate: u32,
    pub wav_precision: super::super::audio_format::WavPrecision,
    pub audiobook: Option<&'a AudiobookOutputConfig>,
    pub measurement: Option<&'a LoudnessMeasurement>,
    pub metadata: Option<&'a Path>,
}

pub(super) fn encode(
    request: EncodeRequest<'_>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let measured = request
        .audiobook
        .filter(|_| request.measurement.is_none())
        .map(|config| {
            measure_loudness(
                request.ffmpeg,
                request.concat_path,
                request.sample_rate,
                config,
                cancellation,
            )
        })
        .transpose()?;
    let measurement = request.measurement.or(measured.as_ref());
    let mut command = Command::new(request.ffmpeg);
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
        ])
        .arg(request.concat_path);
    if let Some(metadata) = request.metadata {
        command.args(["-f", "ffmetadata", "-i"]).arg(metadata);
    }
    let cover = request
        .audiobook
        .and_then(|config| config.metadata.cover.as_ref());
    if let Some(cover) = cover {
        crate::file_inspection::verify_content_revision(
            Path::new(&cover.asset_path),
            &cover.source_revision,
        )?;
        let format = image::ImageReader::open(&cover.asset_path)
            .map_err(|error| error.to_string())?
            .with_guessed_format()
            .map_err(|error| error.to_string())?
            .format();
        if !matches!(
            format,
            Some(image::ImageFormat::Jpeg | image::ImageFormat::Png)
        ) {
            return Err("Audiobook cover must be PNG or JPEG".into());
        }
        command.arg("-i").arg(&cover.asset_path);
    }
    command
        .arg("-filter_complex")
        .arg(filter_graph(
            request.sample_rate,
            request.audiobook.zip(measurement),
        ))
        .args(["-map", "[out]"]);
    if request.metadata.is_some() {
        command.args(["-map_metadata", "1", "-map_chapters", "1"]);
    }
    if cover.is_some() {
        let cover_index = if request.metadata.is_some() {
            "2:v:0"
        } else {
            "1:v:0"
        };
        command.args([
            "-map",
            cover_index,
            "-c:v",
            "copy",
            "-disposition:v:0",
            "attached_pic",
            "-metadata:s:v:0",
            "title=Cover",
        ]);
    }
    match request.format {
        "wav" => {
            command.args(["-c:a", request.wav_precision.codec(), "-f", "wav"]);
        }
        "mp3" => {
            command.args(["-c:a", "libmp3lame", "-q:a", "2", "-f", "mp3"]);
        }
        "m4b" => {
            command.args([
                "-c:a",
                "aac",
                "-b:a",
                "128k",
                "-movflags",
                "+faststart",
                "-f",
                "ipod",
            ]);
        }
        _ => return Err("Speech output format is unsupported".into()),
    }
    let child = command
        .arg(request.output)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start FFmpeg: {error}"))?;
    let outcome = subprocess::wait_with_stderr(child, cancellation, "cancelled", |_| Ok(()))?;
    if outcome.status.success() {
        Ok(())
    } else {
        Err(subprocess::diagnostic_error(
            &outcome.diagnostics,
            "FFmpeg speech encoding failed",
        ))
    }
}

pub(super) fn measure_loudness(
    ffmpeg: &Path,
    concat_path: &Path,
    sample_rate: u32,
    config: &AudiobookOutputConfig,
    cancellation: &CancellationToken,
) -> Result<LoudnessMeasurement, String> {
    let filter = format!(
        "[0:a]aresample={sample_rate},loudnorm=I={}:TP={}:LRA=11:print_format=json[out]",
        config.loudness_lufs, config.true_peak_db,
    );
    let child = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-nostdin",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
        ])
        .arg(concat_path)
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
    serde_json::from_str::<LoudnessMeasurement>(&diagnostics[start..end])
        .map_err(db::json_error)?
        .validate()
}

fn filter_graph(
    sample_rate: u32,
    loudness: Option<(&AudiobookOutputConfig, &LoudnessMeasurement)>,
) -> String {
    if let Some((config, measured)) = loudness {
        format!(
            "[0:a]loudnorm=I={}:TP={}:LRA=11:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true:print_format=summary,aresample={sample_rate},aformat=sample_fmts=flt:channel_layouts=mono[out]",
            config.loudness_lufs, config.true_peak_db, measured.input_i, measured.input_tp,
            measured.input_lra, measured.input_thresh, measured.target_offset,
        )
    } else {
        format!("[0:a]aresample={sample_rate},aformat=sample_fmts=flt:channel_layouts=mono[out]")
    }
}
