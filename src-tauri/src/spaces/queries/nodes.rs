use crate::spaces::db;
use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use tauri::{AppHandle, Runtime};

pub(crate) async fn list_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id, initial_x, initial_y, data FROM spaces_nodes \
         WHERE space_id = ? AND deleted_at IS NULL ORDER BY created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(node_row).collect()
}

#[tauri::command]
pub async fn get_node<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_id: String,
) -> Result<Value, String> {
    let row = sqlx::query(
        "SELECT id, initial_x, initial_y, data FROM spaces_nodes
         WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(db::pool(&app).await?)
    .await
    .map_err(db::sql_error)?
    .ok_or("node-unavailable: resource is unavailable")?;
    node_row(row)
}

fn node_row(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let data_raw: String = row.try_get("data").map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&data_raw).map_err(db::json_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "initialX": row.try_get::<f64, _>("initial_x").map_err(db::sql_error)?,
        "initialY": row.try_get::<f64, _>("initial_y").map_err(db::sql_error)?,
        "data": data,
    }))
}
