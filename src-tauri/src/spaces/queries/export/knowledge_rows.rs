use std::collections::HashSet;

use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use super::parse_json;
use crate::spaces::db;

pub(super) async fn object_types(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id,space_id,name,icon,description,fields,system,created_at,updated_at \
         FROM spaces_object_types WHERE space_id=? AND system=0 ORDER BY created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(serde_json::json!({
                "id": row.try_get::<String,_>("id").map_err(db::sql_error)?,
                "spaceId": row.try_get::<Option<String>,_>("space_id").map_err(db::sql_error)?,
                "name": row.try_get::<String,_>("name").map_err(db::sql_error)?,
                "icon": row.try_get::<String,_>("icon").map_err(db::sql_error)?,
                "description": row.try_get::<String,_>("description").map_err(db::sql_error)?,
                "fields": parse_json(&row, "fields")?, "system": false,
                "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
                "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
            }))
        })
        .collect()
}

pub(super) async fn provenance(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT p.id,p.space_id,p.target_kind,p.target_id,p.source_node_id,p.locator,p.source_revision,p.quote,p.status,p.actor_kind,p.actor_id,p.created_at,p.updated_at \
         FROM spaces_provenance p JOIN spaces_nodes s ON s.id=p.source_node_id AND s.space_id=p.space_id AND s.deleted_at IS NULL \
         WHERE p.space_id=? AND (\
           (p.target_kind='node' AND EXISTS(SELECT 1 FROM spaces_nodes n WHERE n.id=p.target_id AND n.space_id=p.space_id AND n.deleted_at IS NULL)) OR \
           (p.target_kind='relation' AND EXISTS(\
             SELECT 1 FROM spaces_relations r \
             JOIN spaces_nodes rs ON rs.id=r.source_id AND rs.space_id=r.space_id AND rs.deleted_at IS NULL \
             JOIN spaces_nodes rt ON rt.id=r.target_id AND rt.space_id=r.space_id AND rt.deleted_at IS NULL \
             WHERE r.id=p.target_id\
           ))\
         ) ORDER BY p.created_at",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    rows.into_iter().map(|row| Ok(serde_json::json!({
        "id": text(&row,"id")?, "spaceId": text(&row,"space_id")?, "targetKind": text(&row,"target_kind")?,
        "targetId": text(&row,"target_id")?, "sourceNodeId": text(&row,"source_node_id")?, "locator": parse_json(&row,"locator")?,
        "sourceRevision": text(&row,"source_revision")?, "quote": text(&row,"quote")?, "status": text(&row,"status")?,
        "actorKind": text(&row,"actor_kind")?, "actorId": row.try_get::<Option<String>,_>("actor_id").map_err(db::sql_error)?,
        "createdAt": int(&row,"created_at")?, "updatedAt": int(&row,"updated_at")?,
    }))).collect()
}

pub(super) async fn events(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    entities: &HashSet<(String, String)>,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT e.id,e.space_id,e.entity_kind,e.entity_id,e.event_kind,e.actor_kind,e.actor_id,e.payload,e.created_at \
         FROM spaces_knowledge_events e WHERE e.space_id=? ORDER BY e.created_at",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let mut values = Vec::new();
    for row in rows {
        let entity_kind = text(&row, "entity_kind")?;
        let entity_id = text(&row, "entity_id")?;
        if !entities.contains(&(entity_kind.clone(), entity_id.clone())) {
            continue;
        }
        let event_kind = text(&row, "event_kind")?;
        let payload = parse_json(&row, "payload")?;
        values.push(serde_json::json!({
            "id": text(&row,"id")?, "spaceId": text(&row,"space_id")?, "entityKind": entity_kind,
            "entityId": entity_id, "eventKind": event_kind, "actorKind": text(&row,"actor_kind")?,
            "actorId": row.try_get::<Option<String>,_>("actor_id").map_err(db::sql_error)?, "payload": payload,
            "createdAt": int(&row,"created_at")?,
        }));
    }
    Ok(values)
}

fn text(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<String, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn int(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<i64, String> {
    row.try_get(key).map_err(db::sql_error)
}
