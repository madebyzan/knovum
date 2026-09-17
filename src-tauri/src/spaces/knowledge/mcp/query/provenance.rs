use rmcp::ErrorData as McpError;
use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use super::super::{json_text, mcp_error, mcp_sql_error};
use crate::spaces::db;
use crate::spaces::knowledge::mcp::arguments::ProvenanceArgs;

pub(super) async fn run(app: &AppHandle, args: ProvenanceArgs) -> Result<String, McpError> {
    if !matches!(args.target_kind.as_str(), "node" | "relation") {
        return Err(McpError::invalid_params(
            "target_kind must be node or relation",
            None,
        ));
    }
    let pool = db::pool(app).await.map_err(mcp_error)?;
    let rows = sqlx::query(
            "SELECT p.id,p.source_node_id,p.locator,p.source_revision,p.quote,p.status,p.actor_kind,p.actor_id,p.created_at,p.updated_at \
             FROM spaces_provenance p JOIN spaces_semantic_nodes s ON s.id=p.source_node_id AND s.space_id=p.space_id \
             WHERE p.space_id=$1 AND p.target_kind=$2 AND p.target_id=$3 AND ( \
               (p.target_kind='node' AND EXISTS(SELECT 1 FROM spaces_semantic_nodes n WHERE n.id=p.target_id AND n.space_id=p.space_id)) OR \
               (p.target_kind='relation' AND EXISTS(SELECT 1 FROM spaces_relations r WHERE r.id=p.target_id AND r.space_id=p.space_id)) \
             ) ORDER BY p.created_at DESC",
        ).bind(&args.space_id).bind(&args.target_kind).bind(&args.target_id).fetch_all(pool).await.map_err(mcp_sql_error)?;
    let values = rows
        .into_iter()
        .map(|row| {
            let locator: String = row.get("locator");
            let locator =
                serde_json::from_str::<Value>(&locator).map_err(|e| mcp_error(e.to_string()))?;
            Ok(json!({
                "id": row.get::<String,_>("id"),
                "sourceNodeId": row.get::<String,_>("source_node_id"),
                "locator": locator,
                "sourceRevision": row.get::<String,_>("source_revision"),
                "quote": row.get::<String,_>("quote"),
                "status": row.get::<String,_>("status"),
                "actorKind": row.get::<String,_>("actor_kind"),
                "actorId": row.get::<Option<String>,_>("actor_id"),
                "createdAt": row.get::<i64,_>("created_at"),
                "updatedAt": row.get::<i64,_>("updated_at"),
            }))
        })
        .collect::<Result<Vec<Value>, McpError>>()?;
    json_text(values)
}
