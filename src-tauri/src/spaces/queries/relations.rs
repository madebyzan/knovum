use crate::spaces::db;
use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

pub(crate) async fn list_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id, space_id, source_id, target_id, kind, rationale, status, status_before_invalidation, confidence, \
         actor_kind, actor_id, created_at, updated_at FROM spaces_relations \
         WHERE space_id = ? AND status <> 'invalidated' ORDER BY created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(relation_row).collect()
}

fn relation_row(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(serde_json::json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?,
        "spaceId": row.try_get::<String,_>("space_id").map_err(db::sql_error)?,
        "sourceId": row.try_get::<String,_>("source_id").map_err(db::sql_error)?,
        "targetId": row.try_get::<String,_>("target_id").map_err(db::sql_error)?,
        "kind": row.try_get::<String,_>("kind").map_err(db::sql_error)?,
        "rationale": row.try_get::<String,_>("rationale").map_err(db::sql_error)?,
        "status": row.try_get::<String,_>("status").map_err(db::sql_error)?,
        "statusBeforeInvalidation": row.try_get::<Option<String>,_>("status_before_invalidation").map_err(db::sql_error)?,
        "confidence": row.try_get::<Option<f64>,_>("confidence").map_err(db::sql_error)?,
        "actorKind": row.try_get::<String,_>("actor_kind").map_err(db::sql_error)?,
        "actorId": row.try_get::<Option<String>,_>("actor_id").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))
}
