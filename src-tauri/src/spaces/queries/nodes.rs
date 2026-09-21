use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

#[tauri::command]
pub async fn list_nodes<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id, initial_x, initial_y, data FROM spaces_nodes \
         WHERE space_id = ? AND deleted_at IS NULL ORDER BY created_at",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(node_row).collect()
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
