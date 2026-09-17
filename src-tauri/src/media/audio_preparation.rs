use std::{
    path::Path,
    process::{Command, Stdio},
};

use tokio_util::sync::CancellationToken;

use super::{
    subprocess::{diagnostic_error, wait_with_stderr},
    waveform::read_waveform,
    workspace::AnalysisWorkspace,
    TRANSCRIPTION_CANCELLED,
};

const AUDIO_FILTER: &str =
    "[0:a:0]aformat=sample_rates=16000:channel_layouts=mono,asplit=2[transcript][waveform]";

pub(crate) struct PreparedAudio {
    pub waveform: Vec<f32>,
    pub duration_ms: u64,
}

pub fn prepare_audio(
    ffmpeg: &Path,
    source: &Path,
    paths: &AnalysisWorkspace,
    cancellation: &CancellationToken,
) -> Result<PreparedAudio, String> {
    let child = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-progress",
            "pipe:2",
            "-nostats",
            "-y",
            "-i",
        ])
        .arg(source)
        .args([
            "-filter_complex",
            AUDIO_FILTER,
            "-map",
            "[transcript]",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ])
        .arg(&paths.wav)
        .args(["-map", "[waveform]", "-f", "f32le"])
        .arg(&paths.waveform)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch FFmpeg: {error}"))?;
    let mut duration_ms = 0;
    let output = wait_with_stderr(child, cancellation, TRANSCRIPTION_CANCELLED, |line| {
        if let Some(value) = line
            .strip_prefix("out_time_us=")
            .and_then(|value| value.parse::<u64>().ok())
        {
            duration_ms = duration_ms.max(value / 1_000);
        }
        Ok(())
    })?;
    if !output.status.success() {
        return Err(diagnostic_error(
            &output.diagnostics,
            "FFmpeg could not prepare media for transcription",
        ));
    }
    if duration_ms == 0 {
        return Err("FFmpeg did not report a usable media duration".into());
    }
    let waveform = read_waveform(&paths.waveform)?;
    Ok(PreparedAudio {
        waveform: waveform.values,
        duration_ms,
    })
}
