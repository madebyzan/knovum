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

/// The codec of `source`'s first video stream, as FFmpeg names it (`h264`, `hevc`, `prores`…),
/// read from the stream summary FFmpeg prints for its input.
pub(crate) fn video_codec(
    ffmpeg: &Path,
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<String, String> {
    let mut codec = None;
    let child = Command::new(ffmpeg)
        .args(["-nostdin", "-hide_banner", "-loglevel", "info", "-i"])
        .arg(source)
        .args([
            "-map",
            "0:v:0",
            "-c",
            "copy",
            "-frames:v",
            "0",
            "-f",
            "null",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Unable to inspect media video: {error}"))?;
    let exit = subprocess::wait_with_stderr(child, cancellation, "cancelled", |line| {
        if codec.is_none() {
            codec = stream_codec(line, "Video");
        }
        Ok(())
    })?;
    if !exit.status.success() {
        return Err(subprocess::diagnostic_error(
            &exit.diagnostics,
            "Unable to inspect media video",
        ));
    }
    codec.ok_or_else(|| "Media has no video stream".into())
}

/// The codec named by an FFmpeg input summary line for a stream of `kind`, such as
/// `  Stream #0:0[0x1](und): Video: hevc (Main) (hvc1 / 0x31637668), yuv420p…`.
fn stream_codec(line: &str, kind: &str) -> Option<String> {
    let line = line.trim_start();
    if !line.starts_with("Stream #0:") {
        return None;
    }
    let (_, description) = line.split_once(&format!(": {kind}: "))?;
    let codec = description
        .split(|character: char| character.is_whitespace() || character == ',')
        .next()?;
    (!codec.is_empty()).then(|| codec.to_owned())
}

#[cfg(test)]
#[path = "tests/ffmpeg.rs"]
mod tests;
