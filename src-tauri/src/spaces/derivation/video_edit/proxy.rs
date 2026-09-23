use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{blocking, local_resources::tools, media::subprocess, spaces::db};

const CANCELLED: &str = "VIDEO_EDIT_PREPARATION_CANCELLED";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub source_revision: String,
}

fn generation_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub async fn prepare(app: AppHandle, request: PrepareRequest) -> Result<String, String> {
    Uuid::parse_str(&request.id).map_err(|_| "Video preparation ID is invalid")?;
    let (cancellation, _lease) = super::proxy_registry::acquire(request.id)?;
    let pool = db::pool(&app).await?;
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='video' AND deleted_at IS NULL",
    )
    .bind(&request.source_node_id)
    .bind(&request.space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Video source is unavailable")?;
    let data: serde_json::Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if data
        .get("sourceRevision")
        .and_then(serde_json::Value::as_str)
        != Some(request.source_revision.as_str())
    {
        return Err("Video source changed before preparation".into());
    }
    let source = data
        .get("assetPath")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or("Video asset is unavailable")?
        .to_owned();
    blocking::run(move || generate(&app, &source, &request.source_revision, &cancellation)).await
}

fn generate(
    app: &AppHandle,
    source: &str,
    revision: &str,
    cancellation: &CancellationToken,
) -> Result<String, String> {
    let _guard = generation_lock()
        .lock()
        .map_err(|error| error.to_string())?;
    if cancellation.is_cancelled() {
        return Err(CANCELLED.into());
    }
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("video-edit-proxies");
    fs::create_dir_all(&cache).map_err(|error| error.to_string())?;
    let key = Sha256::digest(format!("{source}\0{revision}"));
    let output = cache.join(format!("{key:x}.mp4"));
    if output.metadata().is_ok_and(|metadata| metadata.len() > 0) {
        return Ok(output.to_string_lossy().into_owned());
    }
    let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())
        .ok_or("FFmpeg is required to prepare video editing")?;
    let temporary = cache.join(format!("{}.tmp.mp4", Uuid::new_v4()));
    if let Err(error) = encode_proxy(&ffmpeg, source, &temporary, cancellation) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if let Err(error) = fs::rename(&temporary, &output) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(output.to_string_lossy().into_owned())
}

fn encode_proxy(
    ffmpeg: &PathBuf,
    source: &str,
    output: &PathBuf,
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
            "-map",
            "0:a:0?",
            "-vf",
            "scale=w=960:h=540:force_original_aspect_ratio=decrease:force_divisible_by=2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-threads",
            "2",
            "-crf",
            "25",
            "-g",
            "12",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-b:a",
            "96k",
            "-movflags",
            "+faststart",
        ])
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Unable to start video proxy preparation: {error}"))?;
    let result = subprocess::wait_with_stderr(child, cancellation, CANCELLED, |_| Ok(()))?;
    if !result.status.success() {
        return Err(subprocess::diagnostic_error(
            &result.diagnostics,
            "Video proxy preparation failed",
        ));
    }
    if !output.metadata().is_ok_and(|metadata| metadata.len() > 0) {
        return Err("Video proxy preparation produced no video".into());
    }
    Ok(())
}
