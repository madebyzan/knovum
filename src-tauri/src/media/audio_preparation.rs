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

const SAMPLE_RATE: u64 = 16_000;
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
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
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
        .args(["-map", "[waveform]", "-c:a", "pcm_f32le", "-f", "f32le"])
        .arg(&paths.waveform)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch FFmpeg: {error}"))?;
    let output = wait_with_stderr(child, cancellation, TRANSCRIPTION_CANCELLED, |_| Ok(()))?;
    if !output.status.success() {
        return Err(diagnostic_error(
            &output.diagnostics,
            "FFmpeg could not decode the media audio track",
        ));
    }
    inspect_prepared_audio(paths, cancellation)
}

pub(super) fn inspect_prepared_audio(
    paths: &AnalysisWorkspace,
    cancellation: &CancellationToken,
) -> Result<PreparedAudio, String> {
    if !paths.wav.is_file() {
        return Err("Prepared audio is missing its playback-independent PCM source".into());
    }
    let waveform = read_waveform(&paths.waveform, cancellation)?;
    let duration_ms = waveform
        .sample_count
        .checked_mul(1_000)
        .ok_or("Decoded media duration overflow")?
        .div_ceil(SAMPLE_RATE);
    if duration_ms == 0 {
        return Err("The media audio track contains no decoded samples".into());
    }
    Ok(PreparedAudio {
        waveform: waveform.values,
        duration_ms,
    })
}
