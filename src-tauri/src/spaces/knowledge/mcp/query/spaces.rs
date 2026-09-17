use rmcp::ErrorData as McpError;
use serde_json::json;
use sqlx::Row;
use tauri::AppHandle;

use super::super::{json_text, mcp_error, mcp_sql_error};
use crate::spaces::db;

pub(super) async fn run(app: &AppHandle, space_scope: Option<&str>) -> Result<String, McpError> {
    let pool = db::pool(app).await.map_err(mcp_error)?;
    let rows = if let Some(space_id) = space_scope {
        sqlx::query(
            "SELECT id,kind,title,last_opened_at,updated_at FROM spaces WHERE id=$1 AND archived_at IS NULL",
        ).bind(space_id).fetch_all(pool).await.map_err(mcp_sql_error)?
    } else {
        sqlx::query(
            "SELECT id,kind,title,last_opened_at,updated_at FROM spaces WHERE archived_at IS NULL ORDER BY last_opened_at DESC",
        ).fetch_all(pool).await.map_err(mcp_sql_error)?
    };
    let values = rows.into_iter().map(|row| json!({
            "id": row.get::<String,_>("id"), "kind": row.get::<String,_>("kind"),
            "title": row.get::<String,_>("title"), "lastOpenedAt": row.get::<i64,_>("last_opened_at"),
            "updatedAt": row.get::<i64,_>("updated_at"),
        })).collect::<Vec<_>>();
    json_text(values)
}
