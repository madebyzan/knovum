use crate::{media::subprocess, spaces::speech::contracts::SpeechSegmentRenderOutput};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tokio_util::sync::CancellationToken;

pub(in crate::spaces::speech::project_publication) struct NormalizedSource {
    pub(in crate::spaces::speech::project_publication) path: PathBuf,
    pub(in crate::spaces::speech::project_publication) sample_count: u64,
}

pub(in crate::spaces::speech::project_publication) fn normalize_sources(
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
            let actual = wav_sample_count(
                source,
                sample_rate,
                crate::spaces::speech::audio_format::WavPrecision::Float32,
            )?;
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
            .args(["-c:a", "pcm_f32le", "-f", "wav"])
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
            sample_count: wav_sample_count(
                &path,
                sample_rate,
                crate::spaces::speech::audio_format::WavPrecision::Float32,
            )?,
            path,
        });
    }
    Ok(normalized)
}

pub(in crate::spaces::speech::project_publication) fn wav_sample_count(
    path: &Path,
    expected_rate: u32,
    precision: crate::spaces::speech::audio_format::WavPrecision,
) -> Result<u64, String> {
    let reader = hound::WavReader::open(path).map_err(|error| error.to_string())?;
    let spec = reader.spec();
    let (bits, format) = match precision {
        crate::spaces::speech::audio_format::WavPrecision::Pcm16 => (16, hound::SampleFormat::Int),
        crate::spaces::speech::audio_format::WavPrecision::Pcm24 => (24, hound::SampleFormat::Int),
        crate::spaces::speech::audio_format::WavPrecision::Float32 => {
            (32, hound::SampleFormat::Float)
        }
    };
    if spec.sample_rate != expected_rate
        || spec.channels != 1
        || spec.bits_per_sample != bits
        || spec.sample_format != format
    {
        return Err("Speech WAV does not match the declared mono precision and sample rate".into());
    }
    Ok(u64::from(reader.duration()))
}

pub(in crate::spaces::speech::project_publication) fn write_concat_file(
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
