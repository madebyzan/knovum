use std::collections::HashSet;

use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::spaces::db;

use super::generation::IndexGeneration;

/// Drops LanceDB semantic results whose chunk_id is not present in
/// `spaces_current_knowledge_chunks`, guarding against stale embeddings.
pub(super) async fn filter(
    pool: &SqlitePool,
    generations: &[(&str, &IndexGeneration)],
    values: Vec<Value>,
) -> Result<Vec<Value>, String> {
    if values.is_empty() {
        return Ok(values);
    }
    let chunk_ids = values
        .iter()
        .map(|value| identity(value).map(|(_, chunk_id, _)| chunk_id))
        .collect::<Result<Vec<_>, _>>()?;
    let allowed = generations.iter().map(|(space_id, generation)| serde_json::json!({
        "spaceId": space_id, "generationId": generation.id, "embeddingSpecId": generation.embedding_spec_id,
    })).collect::<Vec<_>>();
    let rows = sqlx::query(
        "WITH allowed AS (SELECT json_extract(value,'$.spaceId') AS space_id, \
           json_extract(value,'$.generationId') AS generation_id, \
           json_extract(value,'$.embeddingSpecId') AS embedding_spec_id FROM json_each($1)) \
         SELECT c.space_id, c.id, c.source_revision FROM allowed a \
         JOIN spaces_current_knowledge_chunks c ON c.space_id=a.space_id \
         JOIN spaces_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         JOIN spaces_chunk_embeddings e ON e.chunk_id=c.id \
           AND e.generation_id=a.generation_id AND e.embedding_spec_id=a.embedding_spec_id \
           AND e.source_revision=c.source_revision \
         WHERE c.id IN (SELECT value FROM json_each($2)) AND n.deleted_at IS NULL",
    )
    .bind(sqlx::types::Json(&allowed))
    .bind(sqlx::types::Json(&chunk_ids))
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let valid = rows
        .into_iter()
        .map(|row| {
            let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
            let chunk_id: String = row.try_get("id").map_err(db::sql_error)?;
            let revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
            Ok((space_id, chunk_id, revision))
        })
        .collect::<Result<HashSet<_>, String>>()?;
    let mut filtered = Vec::new();
    for value in values {
        let (space_id, chunk_id, revision) = identity(&value)?;
        if valid.contains(&(
            space_id.to_owned(),
            chunk_id.to_owned(),
            revision.to_owned(),
        )) {
            filtered.push(value);
        }
    }
    Ok(filtered)
}

fn identity(value: &Value) -> Result<(&str, &str, &str), String> {
    let field = |name| {
        value
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Semantic search result is missing {name}"))
    };
    Ok((
        field("spaceId")?,
        field("chunkId")?,
        field("sourceRevision")?,
    ))
}

#[cfg(test)]
mod tests;
