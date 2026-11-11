use crate::foundation::subprocess;
use std::{
    path::Path,
    process::{Command, Stdio},
};
use tokio_util::sync::CancellationToken;

pub(crate) fn run(
    command: &mut Command,
    cancellation: &CancellationToken,
    message: &str,
) -> Result<(), String> {
    let child = command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start FFmpeg: {error}"))?;
    let outcome = subprocess::wait_with_stderr(child, cancellation, "cancelled", |_| Ok(()))?;
    if outcome.status.success() {
        Ok(())
    } else {
        Err(subprocess::diagnostic_error(&outcome.diagnostics, message))
    }
}

pub(crate) fn duration_us(
    ffmpeg: &Path,
    source: &Path,
    stream: &str,
    cancellation: &CancellationToken,
) -> Result<u64, String> {
    let mut maximum_us = 0_u64;
    let child = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-progress",
            "pipe:1",
            "-nostats",
            "-i",
        ])
        .arg(source)
        .args(["-map", stream, "-c", "copy", "-f", "null", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start FFmpeg media verification: {error}"))?;
    let outcome = subprocess::wait_with_output(
        child,
        cancellation,
        "cancelled",
        |line| {
            if let Some(value) = line
                .strip_prefix("out_time_us=")
                .and_then(|value| value.parse().ok())
            {
                maximum_us = maximum_us.max(value);
            }
            Ok(())
        },
        |_| Ok(()),
    )?;
    if !outcome.status.success() || maximum_us == 0 {
        return Err(subprocess::diagnostic_error(
            &outcome.diagnostics,
            "FFmpeg could not verify media duration",
        ));
    }
    Ok(maximum_us)
}

/// Whether `source` carries an audio stream that decodes. A source without one is an answer, not
/// a failure; anything else FFmpeg reports is. `cancelled` is the error a cancellation reports.
pub(crate) fn has_audio(
    ffmpeg: &Path,
    source: &Path,
    cancellation: &CancellationToken,
    cancelled: &str,
) -> Result<bool, String> {
    let child = Command::new(ffmpeg)
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(source)
        .args([
            "-map",
            "0:a:0?",
            "-vn",
            "-sn",
            "-dn",
            "-frames:a",
            "1",
            "-f",
            "framemd5",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Unable to inspect media audio: {error}"))?;
    let exit = subprocess::wait_with_stderr(child, cancellation, cancelled, |_| Ok(()))?;
    if exit.status.success() {
        return Ok(true);
    }
    let error = subprocess::diagnostic_error(&exit.diagnostics, "Unable to inspect media audio");
    if error.contains("does not contain any stream") {
        Ok(false)
    } else {
        Err(error)
    }
}
