use std::{
    path::Path,
    process::{Command, Stdio},
};

use tokio_util::sync::CancellationToken;

use super::SAMPLE_RATE;
use crate::media::subprocess;

pub(super) fn mux_video(
    ffmpeg: &Path,
    video: &Path,
    audio: &Path,
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(video)
        .arg("-i")
        .arg(audio)
        .args([
            "-filter_complex",
            "[1:a:0]apad[audio]",
            "-map",
            "0:v:0",
            "-map",
            "[audio]",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-shortest",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ])
        .arg(output);
    run_ffmpeg(
        &mut command,
        cancellation,
        "FFmpeg could not publish the dubbed video",
    )
}

pub(super) fn run_ffmpeg(
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

pub(super) fn verified_duration_us(
    ffmpeg: &Path,
    source: &Path,
    stream: &str,
    expected_ms: u64,
    cancellation: &CancellationToken,
) -> Result<u64, String> {
    let actual_us = inspect_duration_us(ffmpeg, source, stream, cancellation)?;
    let expected_us = expected_ms
        .checked_mul(1_000)
        .ok_or("Media duration overflow")?;
    if actual_us.abs_diff(expected_us) > 250_000 {
        return Err("Media duration differs from the analyzed source metadata".into());
    }
    Ok(actual_us)
}

pub(super) fn inspect_duration_us(
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

pub(super) fn samples_for_duration(duration_us: u64) -> Result<u64, String> {
    let samples = u128::from(duration_us)
        .checked_mul(u128::from(SAMPLE_RATE))
        .ok_or("Video duration sample count overflow")?
        .div_ceil(1_000_000);
    u64::try_from(samples).map_err(|_| "Video duration sample count exceeds platform range".into())
}
