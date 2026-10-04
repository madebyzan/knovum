use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::super::{contracts::SpeechSegmentRenderOutput, project_config::AudiobookOutputConfig};
use crate::{media::subprocess, spaces::db};

pub(super) struct NormalizedSource {
    pub(super) path: PathBuf,
    pub(super) sample_count: u64,
}

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

pub(super) fn normalize_sources(
    ffmpeg: &Path,
    sources: &[PathBuf],
    renders: &[SpeechSegmentRenderOutput],
    sample_rate: u32,
    directory: &Path,
    cancellation: &CancellationToken,
) -> Result<Vec<NormalizedSource>, String> {
    let mut normalized = Vec::with_capacity(sources.len());
    for (index, (source, render)) in sources.iter().zip(renders).enumerate() {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        if render.sample_rate == sample_rate {
            let actual = wav_sample_count(source, sample_rate)?;
            if actual != render.sample_count {
                return Err("Speech segment frame count differs from its render metadata".into());
            }
            normalized.push(NormalizedSource {
                path: source.clone(),
                sample_count: actual,
            });
            continue;
        }
        let path = directory.join(format!("normalized-{index}.wav"));
        let part = directory.join(format!("normalized-{index}.wav.part"));
        let child = Command::new(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n", "-i"])
            .arg(source)
            .args(["-map", "0:a:0", "-vn", "-ac", "1", "-ar"])
            .arg(sample_rate.to_string())
            .args(["-c:a", "pcm_s16le", "-f", "wav"])
            .arg(&part)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Failed to start FFmpeg resampling: {error}"))?;
        let outcome = subprocess::wait_with_stderr(child, cancellation, "cancelled", |_| Ok(()))?;
        if !outcome.status.success() {
            return Err(subprocess::diagnostic_error(
                &outcome.diagnostics,
                "FFmpeg speech resampling failed",
            ));
        }
        fs::rename(&part, &path).map_err(|error| error.to_string())?;
        normalized.push(NormalizedSource {
            sample_count: wav_sample_count(&path, sample_rate)?,
            path,
        });
    }
    Ok(normalized)
}

pub(super) fn wav_sample_count(path: &Path, expected_rate: u32) -> Result<u64, String> {
    let reader = hound::WavReader::open(path).map_err(|error| error.to_string())?;
    let spec = reader.spec();
    if spec.sample_rate != expected_rate
        || spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err("Speech assembly input is not mono PCM16 at the target sample rate".into());
    }
    Ok(u64::from(reader.duration()))
}

pub(super) fn write_concat_file(
    directory: &Path,
    sources: &[NormalizedSource],
) -> Result<PathBuf, String> {
    let path = directory.join("inputs.ffconcat.part");
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    file.write_all(b"ffconcat version 1.0\n")
        .map_err(|error| error.to_string())?;
    for source in sources {
        writeln!(file, "file '{}'", ffconcat_path(&source.path)?)
            .map_err(|error| error.to_string())?;
    }
    file.sync_all().map_err(|error| error.to_string())?;
    Ok(path)
}

fn ffconcat_path(path: &Path) -> Result<String, String> {
    let value = path.to_string_lossy();
    if value
        .chars()
        .any(|character| matches!(character, '\0' | '\n' | '\r'))
    {
        return Err("Speech assembly path contains unsupported control characters".into());
    }
    Ok(value.replace('\\', "/").replace('\'', "'\\''"))
}

pub(super) struct EncodeRequest<'a> {
    pub ffmpeg: &'a Path,
    pub concat_path: &'a Path,
    pub output: &'a Path,
    pub format: &'a str,
    pub sample_rate: u32,
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
    match request.format {
        "wav" => {
            command.args(["-c:a", "pcm_s16le", "-f", "wav"]);
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
            "[0:a]loudnorm=I={}:TP={}:LRA=11:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true:print_format=summary,aresample={sample_rate},aformat=sample_fmts=s16:channel_layouts=mono[out]",
            config.loudness_lufs, config.true_peak_db, measured.input_i, measured.input_tp,
            measured.input_lra, measured.input_thresh, measured.target_offset,
        )
    } else {
        format!("[0:a]aresample={sample_rate},aformat=sample_fmts=s16:channel_layouts=mono[out]")
    }
}

pub(super) fn write_chapter_metadata(
    directory: &Path,
    config: &AudiobookOutputConfig,
    chapter_keys: &[Option<String>],
    sources: &[NormalizedSource],
    sample_rate: u32,
) -> Result<PathBuf, String> {
    let path = directory.join("chapters.ffmetadata.part");
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    file.write_all(b";FFMETADATA1\n")
        .map_err(|error| error.to_string())?;
    let mut end_samples = 0_u64;
    let mut source_index = 0_usize;
    for chapter in &config.chapters {
        let start_samples = end_samples;
        while source_index < sources.len()
            && chapter_keys[source_index].as_deref() == Some(chapter.key.as_str())
        {
            end_samples = end_samples
                .checked_add(sources[source_index].sample_count)
                .ok_or("Speech chapter sample count overflow")?;
            source_index += 1;
        }
        writeln!(
            file, "[CHAPTER]\nTIMEBASE=1/{sample_rate}\nSTART={start_samples}\nEND={end_samples}\ntitle={}",
            ffmetadata_value(&chapter.title),
        ).map_err(|error| error.to_string())?;
    }
    if source_index != sources.len() {
        return Err("Speech chapter layout does not cover every rendered segment".into());
    }
    file.sync_all().map_err(|error| error.to_string())?;
    Ok(path)
}

fn ffmetadata_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\n' | '\r' => escaped.push(' '),
            '\\' | '=' | ';' | '#' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    escaped
}
