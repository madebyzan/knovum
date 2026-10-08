use crate::subprocess::{diagnostic_error, wait_with_stderr};
use std::{
    path::Path,
    process::{Command, Stdio},
};
use tokio_util::sync::CancellationToken;

pub(super) fn convert(
    ffmpeg: &Path,
    source: &Path,
    output: &Path,
    cancellation: &CancellationToken,
    interval: (u64, u64),
) -> Result<(), String> {
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let child = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n"])
        .arg("-ss")
        .arg(format!("{:.3}", interval.0 as f64 / 1000.0))
        .arg("-i")
        .arg(source)
        .arg("-t")
        .arg(format!("{:.3}", (interval.1 - interval.0) as f64 / 1000.0))
        .args([
            "-map",
            "0:a:0",
            "-vn",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ])
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start reference audio conversion: {error}"))?;
    let completion = wait_with_stderr(child, cancellation, "cancelled", |_| Ok(()))?;
    if !completion.status.success() {
        return Err(diagnostic_error(
            &completion.diagnostics,
            "Reference audio conversion failed",
        ));
    }
    Ok(())
}

pub(super) fn verify_duration(path: &Path, interval: (u64, u64)) -> Result<(), String> {
    let samples = u64::from(
        hound::WavReader::open(path)
            .map_err(|error| error.to_string())?
            .duration(),
    );
    if samples.abs_diff((interval.1 - interval.0) * 16) > 16 {
        return Err("Reference selection extends beyond its source audio".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
