use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::db;

pub(crate) async fn history(
    app: &AppHandle,
    space_id: &str,
    entity_kind: Option<&str>,
    entity_id: Option<&str>,
    limit: i64,
) -> Result<Value, String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query(
        "SELECT e.id,e.entity_kind,e.entity_id,e.event_kind,e.actor_kind,e.actor_id,e.payload,e.created_at \
         FROM spaces_knowledge_events e WHERE e.space_id=$1 \
         AND ($2 IS NULL OR e.entity_kind=$2) AND ($3 IS NULL OR e.entity_id=$3) \
         AND NOT (e.entity_kind='node' AND EXISTS(SELECT 1 FROM spaces_nodes n \
         WHERE n.id=e.entity_id AND n.kind='credential')) ORDER BY e.created_at DESC LIMIT $4",
    )
    .bind(space_id)
    .bind(entity_kind)
    .bind(entity_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let values = rows.into_iter().map(|row| {
        let payload = parse_json(row.get("payload"))?;
        Ok(json!({
            "id": row.get::<String,_>("id"), "entityKind": row.get::<String,_>("entity_kind"),
            "entityId": row.get::<String,_>("entity_id"), "eventKind": row.get::<String,_>("event_kind"),
            "actorKind": row.get::<String,_>("actor_kind"), "actorId": row.get::<Option<String>,_>("actor_id"),
            "payload": payload, "createdAt": row.get::<i64,_>("created_at"),
        }))
    }).collect::<Result<Vec<_>, String>>()?;
    Ok(Value::Array(values))
}

fn parse_json(raw: String) -> Result<Value, String> {
    serde_json::from_str(&raw).map_err(db::json_error)
}
