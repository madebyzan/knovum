use rmcp::ErrorData as McpError;
use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use super::super::{json_text, mcp_error, mcp_sql_error};
use crate::spaces::db;
use crate::spaces::knowledge::mcp::arguments::HistoryArgs;

pub(super) async fn run(app: &AppHandle, args: HistoryArgs) -> Result<String, McpError> {
    let pool = db::pool(app).await.map_err(mcp_error)?;
    let limit = args.limit.clamp(1, 200);
    let rows = sqlx::query(
            "SELECT e.id,e.entity_kind,e.entity_id,e.event_kind,e.actor_kind,e.actor_id,e.payload,e.created_at FROM spaces_knowledge_events e \
             WHERE e.space_id=$1 AND ($2 IS NULL OR e.entity_kind=$2) AND ($3 IS NULL OR e.entity_id=$3) \
             AND NOT (e.entity_kind='node' AND EXISTS(SELECT 1 FROM spaces_nodes n WHERE n.id=e.entity_id AND n.kind='credential')) \
             ORDER BY e.created_at DESC LIMIT $4",
        ).bind(&args.space_id).bind(&args.entity_kind).bind(&args.entity_id).bind(limit).fetch_all(pool).await.map_err(mcp_sql_error)?;
    let values = rows
        .into_iter()
        .map(|row| {
            let payload: String = row.get("payload");
            let payload =
                serde_json::from_str::<Value>(&payload).map_err(|e| mcp_error(e.to_string()))?;
            Ok(json!({
                "id": row.get::<String,_>("id"),
                "entityKind": row.get::<String,_>("entity_kind"),
                "entityId": row.get::<String,_>("entity_id"),
                "eventKind": row.get::<String,_>("event_kind"),
                "actorKind": row.get::<String,_>("actor_kind"),
                "actorId": row.get::<Option<String>,_>("actor_id"),
                "payload": payload,
                "createdAt": row.get::<i64,_>("created_at"),
            }))
        })
        .collect::<Result<Vec<Value>, McpError>>()?;
    json_text(values)
}
