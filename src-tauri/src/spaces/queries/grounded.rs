use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

#[tauri::command]
pub async fn list_grounded_threads<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id, space_id, scope, scope_node_ids, title, created_at, updated_at \
         FROM spaces_ai_threads WHERE space_id = ? ORDER BY updated_at DESC LIMIT 50",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(thread_row).collect()
}

#[tauri::command]
pub async fn list_grounded_messages<R: Runtime>(
    app: AppHandle<R>,
    thread_id: String,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id, thread_id, role, content, citations, provider, model, created_at \
         FROM spaces_ai_messages WHERE thread_id = ? ORDER BY created_at, id",
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(message_row).collect()
}

pub(crate) fn thread_row(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let scope_node_ids: String = row.try_get("scope_node_ids").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "spaceId": row.try_get::<String, _>("space_id").map_err(db::sql_error)?,
        "scope": row.try_get::<String, _>("scope").map_err(db::sql_error)?,
        "scopeNodeIds": serde_json::from_str::<Value>(&scope_node_ids).map_err(db::json_error)?,
        "title": row.try_get::<String, _>("title").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    }))
}

pub(crate) fn message_row(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let citations: String = row.try_get("citations").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "threadId": row.try_get::<String, _>("thread_id").map_err(db::sql_error)?,
        "role": row.try_get::<String, _>("role").map_err(db::sql_error)?,
        "content": row.try_get::<String, _>("content").map_err(db::sql_error)?,
        "citations": serde_json::from_str::<Value>(&citations).map_err(db::json_error)?,
        "provider": row.try_get::<Option<String>, _>("provider").map_err(db::sql_error)?,
        "model": row.try_get::<Option<String>, _>("model").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
    }))
}
