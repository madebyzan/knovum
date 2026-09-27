use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::db;

pub(crate) async fn list_provenance(
    app: &AppHandle,
    space_id: &str,
    target_kind: &str,
    target_id: &str,
) -> Result<Value, String> {
    if !matches!(target_kind, "node" | "relation") {
        return Err("invalid-target-kind: target kind must be node or relation".into());
    }
    let pool = db::pool(app).await?;
    let rows = sqlx::query(
        "SELECT p.id,p.source_node_id,p.locator,p.source_revision,p.quote,p.status,\
         p.actor_kind,p.actor_id,p.created_at,p.updated_at FROM spaces_provenance p \
         JOIN spaces_semantic_nodes s ON s.id=p.source_node_id AND s.space_id=p.space_id \
         WHERE p.space_id=$1 AND p.target_kind=$2 AND p.target_id=$3 AND (\
         (p.target_kind='node' AND EXISTS(SELECT 1 FROM spaces_semantic_nodes n WHERE n.id=p.target_id AND n.space_id=p.space_id)) OR \
         (p.target_kind='relation' AND EXISTS(SELECT 1 FROM spaces_relations r WHERE r.id=p.target_id AND r.space_id=p.space_id))) \
         ORDER BY p.created_at DESC",
    ).bind(space_id).bind(target_kind).bind(target_id)
        .fetch_all(pool).await.map_err(db::sql_error)?;
    let values = rows.into_iter().map(|row| {
        let locator: String = row.try_get("locator").map_err(db::sql_error)?;
        let locator = serde_json::from_str::<Value>(&locator).map_err(db::json_error)?;
        Ok(json!({
            "id": row.try_get::<String,_>("id").map_err(db::sql_error)?, "sourceNodeId": row.try_get::<String,_>("source_node_id").map_err(db::sql_error)?,
            "locator": locator, "sourceRevision": row.try_get::<String,_>("source_revision").map_err(db::sql_error)?,
            "quote": row.try_get::<String,_>("quote").map_err(db::sql_error)?, "status": row.try_get::<String,_>("status").map_err(db::sql_error)?,
            "actorKind": row.try_get::<String,_>("actor_kind").map_err(db::sql_error)?, "actorId": row.try_get::<Option<String>,_>("actor_id").map_err(db::sql_error)?,
            "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?, "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
        }))
    }).collect::<Result<Vec<_>, String>>()?;
    Ok(Value::Array(values))
}
