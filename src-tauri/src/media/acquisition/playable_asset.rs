use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::{
    contracts::{AcquisitionProgress, MediaKind},
    metadata::YtDlpMetadata,
};
use crate::media::subprocess;

pub(crate) struct Preparation<'a> {
    pub source: &'a Path,
    pub workspace: &'a Path,
    pub ffmpeg: &'a Path,
    pub metadata: &'a YtDlpMetadata,
    pub kind: MediaKind,
    pub extension: &'a str,
    pub progress: &'a Channel<AcquisitionProgress>,
    pub cancellation: &'a CancellationToken,
}

pub(crate) fn prepare(request: Preparation<'_>) -> Result<PathBuf, String> {
    let Preparation {
        source,
        workspace,
        ffmpeg,
        metadata,
        kind,
        extension,
        progress,
        cancellation,
    } = request;
    if metadata.is_playable(kind, extension)? {
        return Ok(source.to_path_buf());
    }
    let target = workspace.join(match kind {
        MediaKind::Video => "playable.mp4",
        MediaKind::Audio => "playable.m4a",
    });
    let mut command = Command::new(ffmpeg);
    command
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-stats_period",
            "1",
            "-progress",
            "pipe:1",
            "-y",
            "-i",
        ])
        .arg(source);
    match kind {
        MediaKind::Video => {
            command.args([
                "-map",
                "0:v:0",
                "-map",
                "0:a:0?",
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-crf",
                "22",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
                "-b:a",
                "160k",
                "-movflags",
                "+faststart",
            ]);
        }
        MediaKind::Audio => {
            command.args(["-map", "0:a:0", "-vn", "-c:a", "aac", "-b:a", "160k"]);
        }
    }
    let child = command
        .arg(&target)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Unable to start bundled FFmpeg: {error}"))?;
    let duration_ms = metadata
        .duration_seconds()
        .and_then(|value| value.checked_mul(1000));
    let result = subprocess::wait_with_output(
        child,
        cancellation,
        "MEDIA_ACQUISITION_CANCELLED",
        |line| report_progress(line, duration_ms, progress),
        |_| Ok(()),
    )?;
    if !result.status.success() {
        return Err(subprocess::diagnostic_error(
            &result.diagnostics,
            "FFmpeg could not prepare a playable media asset",
        ));
    }
    let file = fs::metadata(&target).map_err(|error| error.to_string())?;
    if !file.is_file() || file.len() == 0 {
        return Err("FFmpeg did not produce a playable media asset".into());
    }
    Ok(target)
}

fn report_progress(
    line: &str,
    duration_ms: Option<u64>,
    progress: &Channel<AcquisitionProgress>,
) -> Result<(), String> {
    let Some(processed_us) = line
        .strip_prefix("out_time_ms=")
        .and_then(|value| value.parse::<u64>().ok())
    else {
        return Ok(());
    };
    progress
        .send(AcquisitionProgress {
            stage: "finalizing",
            downloaded_bytes: 0,
            total_bytes: None,
            processed_ms: Some(processed_us / 1000),
            duration_ms,
        })
        .map_err(|error| error.to_string())
}
