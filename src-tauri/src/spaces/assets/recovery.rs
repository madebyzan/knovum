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
    let mut failures = Vec::new();

    for row in rows {
        let node_id: String = match row.try_get("id") {
            Ok(id) => id,
            Err(error) => {
                failures.push(format!("failed to read node id during recovery: {error}"));
                continue;
            }
        };
        let space_id: String = match row.try_get("space_id") {
            Ok(id) => id,
            Err(error) => {
                failures.push(format!(
                    "failed to read space id for node {node_id}: {error}"
                ));
                continue;
            }
        };
        let raw: String = match row.try_get("data") {
            Ok(data) => data,
            Err(error) => {
                failures.push(format!("failed to read data for node {node_id}: {error}"));
                continue;
            }
        };
        let mut data: Value = match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(error) => {
                failures.push(format!(
                    "failed to parse JSON data for node {node_id}: {error}"
                ));
                continue;
            }
        };
        let recovery = match data.get("processingState").and_then(Value::as_str) {
            Some("importing") => {
                recover_interrupted_import(app, &space_id, &node_id, &mut data).await
            }
            Some("processing") => {
                recover_interrupted_processing(app, &space_id, &node_id, &mut data).await
            }
            _ => Ok(()),
        };
        if let Err(error) = recovery {
            failures.push(format!(
                "failed to recover interrupted state for node {node_id}: {error}"
            ));
        }
    }

    collect_cleanup_error(&mut failures, "stale audio", audio::cleanup_stale(app));
    collect_cleanup_error(
        &mut failures,
        "stale media analysis",
        crate::media::workspace::cleanup_stale(app),
    );
    collect_cleanup_error(
        &mut failures,
        "stale media imports",
        crate::spaces::assets::import::media::cleanup_stale_workspaces(app),
    );

    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Space asset recovery failed: {}",
            failures.join("; ")
        ))
    }
}

fn collect_cleanup_error(failures: &mut Vec<String>, label: &str, result: Result<(), String>) {
    if let Err(error) = result {
        failures.push(format!("failed to clean up {label}: {error}"));
    }
}
async fn recover_interrupted_import(
    app: &tauri::AppHandle,
    space_id: &str,
    node_id: &str,
    data: &mut Value,
) -> Result<(), String> {
    if let Some(path) = data.get("assetPath").and_then(Value::as_str) {
        if let Err(error) = remove_owned_partial(app, PathBuf::from(path)) {
            log::warn!("failed to remove partial asset {path}: {error}");
        }
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
