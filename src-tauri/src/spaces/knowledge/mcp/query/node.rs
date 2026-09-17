use rmcp::ErrorData as McpError;
use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use super::super::{json_text, mcp_error, mcp_sql_error};
use crate::spaces::db;
use crate::spaces::knowledge::mcp::arguments::NodeArgs;

pub(super) async fn run(app: &AppHandle, args: NodeArgs) -> Result<String, McpError> {
    let pool = db::pool(app).await.map_err(mcp_error)?;
    let row = sqlx::query("SELECT id,kind,data,created_at,updated_at FROM spaces_semantic_nodes WHERE space_id=$1 AND id=$2")
            .bind(&args.space_id).bind(&args.node_id).fetch_optional(pool).await.map_err(mcp_sql_error)?
            .ok_or_else(|| McpError::invalid_params("node does not exist in the requested Space or is not exposed to MCP", None))?;
    let data: String = row.get("data");
    let data = serde_json::from_str::<Value>(&data).map_err(|e| mcp_error(e.to_string()))?;
    let chunks = sqlx::query(
            "SELECT c.id,c.kind,c.source_revision,c.text,c.locator,c.updated_at FROM spaces_current_knowledge_chunks c \
             JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
             WHERE c.space_id=$1 AND c.node_id=$2 \
             ORDER BY c.updated_at DESC LIMIT 20",
        ).bind(&args.space_id).bind(&args.node_id).fetch_all(pool).await.map_err(mcp_sql_error)?;
    let chunks = chunks.into_iter().map(|chunk| {
            let locator: String = chunk.get("locator");
            Ok(json!({
                "id": chunk.get::<String,_>("id"),
                "kind": chunk.get::<String,_>("kind"),
                "sourceRevision": chunk.get::<String,_>("source_revision"),
                "text": chunk.get::<String,_>("text"),
                "locator": serde_json::from_str::<Value>(&locator).map_err(|e| mcp_error(e.to_string()))?,
                "updatedAt": chunk.get::<i64,_>("updated_at"),
            }))
        }).collect::<Result<Vec<Value>, McpError>>()?;
    json_text(
        json!({"id": row.get::<String,_>("id"), "kind": row.get::<String,_>("kind"),
            "data": data, "chunks": chunks,
            "createdAt": row.get::<i64,_>("created_at"), "updatedAt": row.get::<i64,_>("updated_at")}),
    )
}
