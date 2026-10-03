use std::collections::HashSet;

use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::spaces::db;

use super::generation::IndexGeneration;

/// Drops LanceDB semantic results whose chunk_id is not present in
/// `spaces_current_knowledge_chunks`, guarding against stale embeddings.
pub(super) async fn filter(
    pool: &SqlitePool,
    default_space_id: &str,
    generation: &IndexGeneration,
    values: Vec<Value>,
) -> Result<Vec<Value>, String> {
    if values.is_empty() {
        return Ok(values);
    }
    let (chunk_ids, space_ids): (Vec<_>, Vec<_>) = values
        .iter()
        .filter_map(|value| {
            let chunk_id = value.get("chunkId").and_then(Value::as_str)?;
            let space_id = value
                .get("spaceId")
                .and_then(Value::as_str)
                .unwrap_or(default_space_id);
            Some((chunk_id, space_id))
        })
        .unzip();
    if chunk_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        "SELECT c.space_id, c.id, c.source_revision FROM spaces_current_knowledge_chunks c \
         JOIN spaces_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         JOIN spaces_chunk_embeddings e ON e.chunk_id=c.id \
           AND e.generation_id=$3 AND e.embedding_spec_id=$4 \
           AND e.source_revision=c.source_revision \
         WHERE c.space_id IN (SELECT value FROM json_each($1)) \
           AND c.id IN (SELECT value FROM json_each($2)) AND n.deleted_at IS NULL",
    )
    .bind(serde_json::to_string(&space_ids).map_err(db::json_error)?)
    .bind(serde_json::to_string(&chunk_ids).map_err(db::json_error)?)
    .bind(&generation.id)
    .bind(&generation.embedding_spec_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let valid = rows
        .into_iter()
        .map(|row| {
            let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
            let chunk_id: String = row.try_get("id").map_err(db::sql_error)?;
            let revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
            Ok(format!("{space_id}:{chunk_id}:{revision}"))
        })
        .collect::<Result<HashSet<_>, String>>()?;
    Ok(values
        .into_iter()
        .filter(|value| {
            value
                .get("chunkId")
                .and_then(Value::as_str)
                .is_some_and(|chunk_id| {
                    let Some(revision) = value.get("sourceRevision").and_then(Value::as_str) else {
                        return false;
                    };
                    let space_id = value
                        .get("spaceId")
                        .and_then(Value::as_str)
                        .unwrap_or(default_space_id);
                    valid.contains(&format!("{space_id}:{chunk_id}:{revision}"))
                })
        })
        .collect())
}

#[cfg(test)]
mod tests;
