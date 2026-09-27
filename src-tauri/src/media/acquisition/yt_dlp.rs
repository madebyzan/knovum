use std::{
    path::Path,
    process::{Command, Stdio},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::contracts::AcquisitionProgress;
use crate::media::subprocess;

pub(crate) const ACQUISITION_CANCELLED: &str = "media-acquisition-cancelled";
const FORMAT: &str = "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/bestvideo+bestaudio/best";
const PROGRESS_PREFIX: &str = "knovum-progress:";

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AcquisitionError {
    Unsupported,
    Failed(String),
}

pub(crate) fn acquire(
    executable: &Path,
    deno: &Path,
    ffmpeg: &Path,
    url: &str,
    workspace: &Path,
    progress: &Channel<AcquisitionProgress>,
    cancellation: &CancellationToken,
) -> Result<(), AcquisitionError> {
    let js_runtime = format!("deno:{}", deno.to_string_lossy());
    let output = workspace.join("media.%(ext)s");
    let thumbnail = workspace.join("thumbnail.%(ext)s");
    let metadata = workspace.join("metadata.json");
    let mut command = Command::new(executable);
    command
        .args([
            "--no-config",
            "--no-plugin-dirs",
            "--no-remote-components",
            "--no-cache-dir",
            "--no-js-runtimes",
            "--js-runtimes",
        ])
        .arg(js_runtime)
        .args([
            "--no-playlist",
            "--progress",
            "--newline",
            "--progress-delta",
            "1",
            "--no-colors",
            "--socket-timeout",
            "15",
            "--retries",
            "2",
            "--extractor-retries",
            "2",
            "--format",
            FORMAT,
            "--format-sort",
            "+codec:avc:m4a",
            "--merge-output-format",
            "mp4",
            "--write-thumbnail",
            "--convert-thumbnails",
            "jpg",
            "--ffmpeg-location",
        ])
        .arg(ffmpeg)
        .arg("--output")
        .arg(output)
        .arg("--output")
        .arg(format!("thumbnail:{}", thumbnail.to_string_lossy()))
        .args(["--print-to-file", "after_move:%()j"])
        .arg(metadata)
        .args([
            "--progress-template",
            "download:knovum-progress:%(progress.downloaded_bytes)s:%(progress.total_bytes_estimate)s",
            "--",
            url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command
        .spawn()
        .map_err(|error| failed(format!("Unable to start prepared yt-dlp: {error}")))?;
    let output = subprocess::wait_with_output(
        child,
        cancellation,
        ACQUISITION_CANCELLED,
        |line| report_progress(line, progress),
        |_| Ok(()),
    )
    .map_err(failed)?;
    if output.status.success() {
        return Ok(());
    }
    if unsupported_output(&output.diagnostics.join("\n")) {
        return Err(AcquisitionError::Unsupported);
    }
    Err(failed(subprocess::diagnostic_error(
        &output.diagnostics,
        "yt-dlp media download failed",
    )))
}

fn report_progress(line: &str, progress: &Channel<AcquisitionProgress>) -> Result<(), String> {
    let Some(value) = line.strip_prefix(PROGRESS_PREFIX) else {
        return Ok(());
    };
    let mut fields = value.splitn(2, ':');
    progress
        .send(AcquisitionProgress {
            stage: "downloading",
            downloaded_bytes: parse_bytes(fields.next()).unwrap_or(0),
            total_bytes: parse_bytes(fields.next()),
            processed_ms: None,
            duration_ms: None,
        })
        .map_err(|error| error.to_string())
}

fn parse_bytes(value: Option<&str>) -> Option<u64> {
    value?.trim().parse().ok()
}

pub(crate) fn unsupported_output(stderr: &str) -> bool {
    stderr.lines().any(|line| {
        line.contains("Unsupported URL:")
            || line.contains("No video could be found in this webpage")
    })
}

fn failed(message: impl Into<String>) -> AcquisitionError {
    AcquisitionError::Failed(message.into())
}
