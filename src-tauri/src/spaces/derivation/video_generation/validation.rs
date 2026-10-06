use std::{
    path::Path,
    process::{Command, Stdio},
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

/// Decode the submitted artifact before publication; a filename is not a media contract.
pub(super) async fn validate(
    app: &AppHandle,
    path: &str,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let ffmpeg = require_decoder(app)?;
    let path = Path::new(path).to_owned();
    use tokio::io::AsyncReadExt;
    let mut header = [0u8; 8];
    tokio::fs::File::open(&path)
        .await
        .map_err(|error| error.to_string())?
        .read_exact(&mut header)
        .await
        .map_err(|error| format!("Generated video header is invalid: {error}"))?;
    if &header[4..8] != b"ftyp" {
        return Err("Generated video must be an MP4 container".into());
    }
    let cancellation = cancellation.clone();
    crate::blocking::run(move || {
        let child = Command::new(ffmpeg)
            .args(["-nostdin", "-v", "error", "-xerror", "-i"])
            .arg(path)
            .args(["-map", "0:v:0", "-frames:v", "1", "-f", "null", "-"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Could not launch generated video validation: {error}"))?;
        let output = crate::subprocess::wait_with_stderr(
            child,
            &cancellation,
            "Video validation was cancelled",
            |_| Ok(()),
        )?;
        if !output.status.success() {
            return Err(crate::subprocess::diagnostic_error(
                &output.diagnostics,
                "Generated video is not decodable",
            ));
        }
        Ok(())
    })
    .await
}

pub(super) fn require_decoder(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    crate::local_resources::tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", crate::local_resources::tools::ffmpeg_name())?
        .ok_or_else(|| "local-tool-unavailable: FFmpeg is required to validate generated video and is not installed".into())
}
