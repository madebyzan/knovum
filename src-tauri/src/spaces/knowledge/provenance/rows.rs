use serde_json::Value;
use sqlx::Row;

use crate::spaces::db;

pub(super) fn provenance_row(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let locator: String = row.try_get("locator").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?, "spaceId": row.try_get::<String,_>("space_id").map_err(db::sql_error)?,
        "targetKind": row.try_get::<String,_>("target_kind").map_err(db::sql_error)?, "targetId": row.try_get::<String,_>("target_id").map_err(db::sql_error)?,
        "sourceNodeId": row.try_get::<String,_>("source_node_id").map_err(db::sql_error)?, "locator": serde_json::from_str::<Value>(&locator).map_err(db::json_error)?,
        "sourceRevision": row.try_get::<String,_>("source_revision").map_err(db::sql_error)?, "quote": row.try_get::<String,_>("quote").map_err(db::sql_error)?,
        "status": row.try_get::<String,_>("status").map_err(db::sql_error)?, "actorKind": row.try_get::<String,_>("actor_kind").map_err(db::sql_error)?,
        "actorId": row.try_get::<Option<String>,_>("actor_id").map_err(db::sql_error)?, "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))
}

pub(super) fn event_row(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let payload: String = row.try_get("payload").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?, "spaceId": row.try_get::<String,_>("space_id").map_err(db::sql_error)?,
        "entityKind": row.try_get::<String,_>("entity_kind").map_err(db::sql_error)?, "entityId": row.try_get::<String,_>("entity_id").map_err(db::sql_error)?,
        "eventKind": row.try_get::<String,_>("event_kind").map_err(db::sql_error)?, "actorKind": row.try_get::<String,_>("actor_kind").map_err(db::sql_error)?,
        "actorId": row.try_get::<Option<String>,_>("actor_id").map_err(db::sql_error)?, "payload": serde_json::from_str::<Value>(&payload).map_err(db::json_error)?,
        "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
    }))
}
