use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
};

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{media_source, PrepareVideoEditProxyRequest};
use crate::{foundation::blocking, foundation::subprocess, local_resources::tools};

const CANCELLED: &str = "VIDEO_EDIT_PREPARATION_CANCELLED";
/// Identity of the proxy recipe below; changing the recipe invalidates previously cached proxies.
/// The proxy carries no sound: source audio is always read from the source itself.
const RECIPE: &str = "h264-1920-fast-g12-silent-v3";

fn generation_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// A decodable stand-in for a source the WebView cannot decode itself. Sources it can decode are
/// previewed directly and never come here.
pub async fn prepare(
    app: AppHandle,
    request: PrepareVideoEditProxyRequest,
) -> Result<String, String> {
    Uuid::parse_str(&request.id).map_err(|_| "Video preparation ID is invalid")?;
    let (cancellation, _lease) = super::proxy_registry::acquire(request.id)?;
    let source = media_source::resolve(&app, &request.source, &["video"]).await?;
    let output = media_source::cache_path(
        &app,
        "video-edit-proxies",
        &source,
        &request.source.source_revision,
        RECIPE,
    )?
    .with_extension("mp4");
    blocking::run(move || generate(&app, &source, &output, &cancellation)).await
}

fn generate(
    app: &AppHandle,
    source: &str,
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<String, String> {
    let _guard = generation_lock()
        .lock()
        .map_err(|error| error.to_string())?;
    if cancellation.is_cancelled() {
        return Err(CANCELLED.into());
    }
    if output.metadata().is_ok_and(|metadata| metadata.len() > 0) {
        return Ok(output.to_string_lossy().into_owned());
    }
    let ffmpeg =
        tools::resolve_ffmpeg(app)?.ok_or("FFmpeg is required to prepare video editing")?;
    let temporary = output.with_extension(format!("{}.tmp.mp4", Uuid::new_v4()));
    if let Err(error) = encode_proxy(&ffmpeg, source, &temporary, cancellation) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if let Err(error) = fs::rename(&temporary, output) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(output.to_string_lossy().into_owned())
}

fn encode_proxy(
    ffmpeg: &Path,
    source: &str,
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let child = Command::new(ffmpeg)
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            source,
            "-map",
            "0:v:0",
            "-an",
            "-vf",
            "scale=w='min(1920,iw)':h='min(1920,ih)':force_original_aspect_ratio=decrease:force_divisible_by=2",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-threads",
            "0",
            "-crf",
            "22",
            "-g",
            "12",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
        ])
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Unable to start video proxy preparation: {error}"))?;
    let process_output = subprocess::wait_with_stderr(child, cancellation, CANCELLED, |_| Ok(()))?;
    if !process_output.status.success() {
        return Err(subprocess::diagnostic_error(
            &process_output.diagnostics,
            "Video proxy preparation failed",
        ));
    }
    if !output.metadata().is_ok_and(|metadata| metadata.len() > 0) {
        return Err("Video proxy preparation produced no video".into());
    }
    Ok(())
}
