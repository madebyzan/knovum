use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::db;

pub(crate) async fn node(app: &AppHandle, space_id: &str, node_id: &str) -> Result<Value, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query(
        "SELECT id,kind,data,created_at,updated_at FROM spaces_semantic_nodes \
         WHERE space_id=$1 AND id=$2",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("node-unavailable: node does not exist in the requested Space")?;
    let data = parse_json(row.get("data"))?;
    let chunks = sqlx::query(
        "SELECT c.id,c.kind,c.source_revision,c.text,c.locator,c.updated_at \
         FROM spaces_current_knowledge_chunks c JOIN spaces_semantic_nodes n \
         ON n.id=c.node_id AND n.space_id=c.space_id WHERE c.space_id=$1 AND c.node_id=$2 \
         ORDER BY c.updated_at DESC LIMIT 20",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let chunks = chunks
        .into_iter()
        .map(|chunk| {
            Ok(json!({
                "id": chunk.get::<String,_>("id"), "kind": chunk.get::<String,_>("kind"),
                "sourceRevision": chunk.get::<String,_>("source_revision"),
                "text": chunk.get::<String,_>("text"),
                "locator": parse_json(chunk.get("locator"))?,
                "updatedAt": chunk.get::<i64,_>("updated_at"),
            }))
        })
        .collect::<Result<Vec<Value>, String>>()?;
    Ok(json!({
        "id": row.get::<String,_>("id"), "kind": row.get::<String,_>("kind"),
        "data": data, "chunks": chunks, "createdAt": row.get::<i64,_>("created_at"),
        "updatedAt": row.get::<i64,_>("updated_at"),
    }))
}

fn parse_json(raw: String) -> Result<Value, String> {
    serde_json::from_str(&raw).map_err(db::json_error)
}
