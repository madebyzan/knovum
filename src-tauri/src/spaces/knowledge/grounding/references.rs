use std::collections::HashSet;

use serde_json::Value;
use sqlx::Row;

use crate::spaces::db;

/// Converts matching source citations into immutable historical snapshots.
pub(in crate::spaces) async fn detach_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
) -> Result<(), String> {
    if node_ids.is_empty() {
        return Ok(());
    }
    detach_matching_citations(tx, space_id, "nodeId", node_ids).await
}

pub(in crate::spaces) async fn detach_chunks(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    chunk_ids: &[String],
) -> Result<(), String> {
    if chunk_ids.is_empty() {
        return Ok(());
    }
    detach_matching_citations(tx, space_id, "chunkId", chunk_ids).await
}

async fn detach_matching_citations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    field: &str,
    ids: &[String],
) -> Result<(), String> {
    let path = match field {
        "nodeId" => "$.nodeId",
        "chunkId" => "$.chunkId",
        _ => return Err("Unsupported Grounded citation reference field".into()),
    };
    let ids_json = serde_json::to_string(ids).map_err(db::json_error)?;
    let rows = sqlx::query(
        "SELECT m.id,m.content FROM spaces_conversation_messages m \
         JOIN spaces_conversations t ON t.id=m.conversation_id WHERE t.space_id=$1 AND EXISTS ( \
           SELECT 1 FROM json_each(m.content,'$.citations') c \
           WHERE COALESCE(json_extract(c.value,'$.state'),'current') <> 'detached' \
             AND json_extract(c.value,$2) IN (SELECT value FROM json_each($3)) \
         )",
    )
    .bind(space_id)
    .bind(path)
    .bind(&ids_json)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let ids = ids.iter().map(String::as_str).collect::<HashSet<_>>();
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row.try_get("content").map_err(db::sql_error)?;
        let mut content: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let Some(citation_records) = content.get_mut("citations").and_then(Value::as_array_mut)
        else {
            continue;
        };
        let mut changed = false;
        for citation in citation_records {
            if citation.get("state").and_then(Value::as_str) == Some("detached") {
                continue;
            }
            if !citation
                .get(field)
                .and_then(Value::as_str)
                .is_some_and(|id| ids.contains(id))
            {
                continue;
            }
            if let Some(object) = citation.as_object_mut() {
                object.insert("state".into(), Value::String("detached".into()));
                object.remove("nodeId");
                object.remove("chunkId");
                changed = true;
            }
        }
        if !changed {
            continue;
        }
        sqlx::query("UPDATE spaces_conversation_messages SET content=$1 WHERE id=$2")
            .bind(serde_json::to_string(&content).map_err(db::json_error)?)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}
