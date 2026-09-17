use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

#[derive(serde::Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeletedNodeSummary {
    id: String,
    space_id: String,
    space_title: String,
    kind: String,
    title: String,
    deleted_at: i64,
}

#[tauri::command]
pub async fn list_nodes<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id, initial_x, initial_y, data FROM spaces_nodes \
         WHERE space_id = ? AND deleted_at IS NULL \
         ORDER BY CASE kind WHEN 'section' THEN 0 ELSE 1 END, created_at",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(node_row).collect()
}

#[tauri::command]
pub async fn list_recently_deleted_items<R: Runtime>(
    app: AppHandle<R>,
    space_id: Option<String>,
) -> Result<Vec<DeletedNodeSummary>, String> {
    let pool = db::pool(&app).await?;
    sqlx::query_as(
        "SELECT n.id, n.space_id, s.title AS space_title, n.kind, \
         json_extract(n.data, '$.title') AS title, n.deleted_at \
         FROM spaces_nodes n JOIN spaces s ON s.id = n.space_id \
         WHERE (? IS NULL OR n.space_id = ?) AND n.deleted_at IS NOT NULL AND n.kind <> 'section' \
         ORDER BY n.deleted_at DESC, n.id",
    )
    .bind(&space_id)
    .bind(&space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
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
