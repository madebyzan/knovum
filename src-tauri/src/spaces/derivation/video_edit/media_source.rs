use std::{fs, path::PathBuf};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use super::VideoEditMediaSource;
use crate::spaces::db;

/// The managed file behind one ready revision of a material. Every read the editor makes of a
/// source — its proxy, its audio, its waveform — goes through here, so none can reach a file the
/// Space does not manage, a revision that has since changed, or media still being processed.
pub(super) async fn resolve(
    app: &AppHandle,
    source: &VideoEditMediaSource,
    kinds: &[&str],
) -> Result<String, String> {
    let pool = db::pool(app).await?;
    let (kind, encoded) = sqlx::query_as::<_, (String, String)>(
        "SELECT kind, data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(&source.source_node_id)
    .bind(&source.space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Media source is unavailable")?;
    if !kinds.contains(&kind.as_str()) {
        return Err("Media source is unavailable".into());
    }
    let data: serde_json::Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if data
        .get("sourceRevision")
        .and_then(serde_json::Value::as_str)
        != Some(source.source_revision.as_str())
        || data.get("processingState").is_some()
    {
        return Err("Media source changed before it could be read".into());
    }
    let path = data
        .get("assetPath")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or("Media asset is unavailable")?;
    let path = crate::spaces::assets::authorize_spaces_asset(app, &PathBuf::from(path))?;
    if !path.is_file() {
        return Err("Media source must be a managed file".into());
    }
    Ok(path.to_string_lossy().into_owned())
}

/// Where a derived file of one source revision is kept. The recipe is part of the identity, so
/// changing how the file is made never serves one made the old way.
pub(super) fn cache_path(
    app: &AppHandle,
    directory: &str,
    source: &str,
    revision: &str,
    recipe: &str,
) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join(directory);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let key = Sha256::digest(format!("{source}\0{revision}\0{recipe}"));
    Ok(directory.join(hex::encode(key)))
}

/// Publishes a derived file whole: a reader sees it complete or not at all.
pub(super) fn store(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&temporary, bytes)
        .and_then(|()| fs::rename(&temporary, path))
        .map_err(|error| {
            let _ = fs::remove_file(&temporary);
            error.to_string()
        })
}
