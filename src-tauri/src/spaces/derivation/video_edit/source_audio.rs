use crate::foundation::subprocess;
use std::{
    path::Path,
    process::{Command, Stdio},
};
use tokio_util::sync::CancellationToken;

pub(super) fn inspect(
    ffmpeg: &Path,
    source: &str,
    cancellation: &CancellationToken,
) -> Result<bool, String> {
    let child = Command::new(ffmpeg)
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            source,
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
        .map_err(|error| format!("Unable to inspect video audio: {error}"))?;
    let exit = subprocess::wait_with_stderr(child, cancellation, super::CANCELLED, |_| Ok(()))?;
    if exit.status.success() {
        return Ok(true);
    }
    let error = subprocess::diagnostic_error(&exit.diagnostics, "Unable to inspect video audio");
    if error.contains("does not contain any stream") {
        Ok(false)
    } else {
        Err(error)
    }
}
