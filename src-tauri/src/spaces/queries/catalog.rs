use crate::command_error::CommandError;
use serde::Serialize;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpaceSummary {
    id: String,
    kind: String,
    title: String,
    archived_at: Option<i64>,
    favorite: bool,
    thumbnail_path: Option<String>,
    last_opened_at: i64,
    created_at: i64,
    updated_at: i64,
    node_count: i64,
}

#[tauri::command]
pub async fn list_spaces<R: Runtime>(app: AppHandle<R>) -> Result<Vec<SpaceSummary>, CommandError> {
    let pool = db::pool(&app).await?;
    Ok((sqlx::query_as(
        "SELECT spaces.id, spaces.kind, spaces.title, archived_at, favorite, thumbnail_path, \
         last_opened_at, spaces.created_at, spaces.updated_at, \
         COUNT(spaces_nodes.id) AS node_count \
         FROM spaces LEFT JOIN spaces_nodes ON spaces_nodes.space_id = spaces.id AND spaces_nodes.deleted_at IS NULL \
         GROUP BY spaces.id ORDER BY CASE spaces.kind WHEN 'inbox' THEN 0 ELSE 1 END, spaces.created_at",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error))?)
}

#[tauri::command]
pub async fn list_credential_node_ids<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<String>, CommandError> {
    let pool = db::pool(&app).await?;
    let rows =
        sqlx::query("SELECT id FROM spaces_nodes WHERE space_id = ? AND kind = 'credential'")
            .bind(space_id)
            .fetch_all(pool)
            .await
            .map_err(db::sql_error)?;
    Ok((rows
        .into_iter()
        .map(|row| row.try_get::<String, _>("id").map_err(db::sql_error))
        .collect::<Result<_, String>>())?)
}
