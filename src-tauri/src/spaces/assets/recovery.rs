use std::path::PathBuf;

use serde_json::Value;
use sqlx::Row;

use super::{asset_directory, audio, enrichment, transfer::remove_path};
use crate::spaces::db;

pub async fn reconcile(app: &tauri::AppHandle) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query(
        "SELECT id, space_id, data FROM spaces_nodes \
         WHERE deleted_at IS NULL AND json_extract(data, '$.processingState') IN ('importing','processing')",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;

    for row in rows {
        let node_id: String = row.try_get("id").map_err(db::sql_error)?;
        let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let mut data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        match data.get("processingState").and_then(Value::as_str) {
            Some("importing") => {
                recover_interrupted_import(app, &space_id, &node_id, &mut data).await?
            }
            Some("processing") => {
                recover_interrupted_processing(app, &space_id, &node_id, &mut data).await?
            }
            _ => return Err("Interrupted asset query returned an invalid processing state".into()),
        }
    }
    audio::cleanup_stale(app)?;
    crate::media::workspace::cleanup_stale(app)?;
    super::import::media::cleanup_stale_workspaces(app)?;
    Ok(())
}

async fn recover_interrupted_import(
    app: &tauri::AppHandle,
    space_id: &str,
    node_id: &str,
    data: &mut Value,
) -> Result<(), String> {
    if let Some(path) = data.get("assetPath").and_then(Value::as_str) {
        remove_owned_partial(app, PathBuf::from(path))?;
    }
    enrichment::persistence::persist_processing_state(
        app, space_id, node_id, data, "failed", None, None,
    )
    .await
}

async fn recover_interrupted_processing(
    app: &tauri::AppHandle,
    space_id: &str,
    node_id: &str,
    data: &mut Value,
) -> Result<(), String> {
    enrichment::persistence::persist_processing_state(
        app, space_id, node_id, data, "failed", None, None,
    )
    .await
}

fn remove_owned_partial(app: &tauri::AppHandle, path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let root = asset_directory(app)?;
    if path.parent() != Some(root.as_path()) {
        return Err("Interrupted import path is outside the Space asset directory".to_owned());
    }
    remove_path(&path)
}
