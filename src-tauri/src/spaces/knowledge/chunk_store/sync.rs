use std::collections::{HashMap, VecDeque};

use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use super::{
    super::{grounding, indexing::KnowledgeChunkInput},
    write::{insert, update_metadata},
};
use crate::spaces::db;

type IdentityKey = (Option<String>, String, String, String);

struct ExistingChunk {
    id: String,
    locator: Value,
    metadata: Value,
}

/// Synchronizes an origin's passages (or one metadata role within that origin) while preserving
/// the chunk identities of unchanged passages so their grounded citations and embeddings survive.
pub(super) async fn sync_chunks(
    tx: &mut Transaction<'_, Sqlite>,
    (space_id, node_id): (&str, &str),
    (origin, role): (&str, Option<&str>),
    chunks: &[KnowledgeChunkInput],
    now: i64,
) -> Result<(), String> {
    let mut existing = load_existing(tx, (space_id, node_id), origin, role).await?;
    let mut to_insert = Vec::new();
    let empty_metadata = Value::Object(Default::default());

    for chunk in chunks {
        let source_revision = chunk
            .locator
            .get("sourceRevision")
            .and_then(Value::as_str)
            .ok_or_else(|| "Knowledge chunk locator is missing sourceRevision".to_owned())?;
        let metadata = chunk.metadata.as_ref().unwrap_or(&empty_metadata);
        let key = (
            chunk_role(metadata),
            chunk.kind.clone(),
            source_revision.to_owned(),
            chunk.text.trim().to_owned(),
        );
        if let Some(matched) = take_matching(&mut existing, &key, &chunk.locator) {
            if matched.metadata != *metadata {
                update_metadata(tx, &matched.id, metadata, now).await?;
            }
        } else {
            to_insert.push(chunk);
        }
    }

    let stale_ids = existing
        .into_values()
        .flat_map(|bucket| bucket.into_iter().map(|row| row.id))
        .collect::<Vec<_>>();
    delete_by_ids(tx, (space_id, node_id), &stale_ids).await?;
    for chunk in to_insert {
        insert(tx, space_id, node_id, origin, chunk, now).await?;
    }
    Ok(())
}

pub(super) async fn delete_all(
    tx: &mut Transaction<'_, Sqlite>,
    (space_id, node_id): (&str, &str),
    origin: &str,
    role: Option<&str>,
) -> Result<(), String> {
    let old_ids = sqlx::query_scalar::<_, String>(
        "SELECT id FROM spaces_knowledge_chunks WHERE node_id=$1 AND space_id=$2 AND origin=$3 \
         AND ($4 IS NULL OR json_extract(metadata,'$.role')=$4)",
    )
    .bind(node_id)
    .bind(space_id)
    .bind(origin)
    .bind(role)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    delete_by_ids(tx, (space_id, node_id), &old_ids).await
}

async fn load_existing(
    tx: &mut Transaction<'_, Sqlite>,
    (space_id, node_id): (&str, &str),
    origin: &str,
    role: Option<&str>,
) -> Result<HashMap<IdentityKey, VecDeque<ExistingChunk>>, String> {
    let rows = sqlx::query(
        "SELECT id, kind, source_revision, text, locator, metadata \
         FROM spaces_knowledge_chunks \
         WHERE node_id=$1 AND space_id=$2 AND origin=$3 \
           AND ($4 IS NULL OR json_extract(metadata,'$.role')=$4) \
         ORDER BY created_at, rowid",
    )
    .bind(node_id)
    .bind(space_id)
    .bind(origin)
    .bind(role)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;

    let mut grouped: HashMap<IdentityKey, VecDeque<ExistingChunk>> = HashMap::new();
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let kind: String = row.try_get("kind").map_err(db::sql_error)?;
        let source_revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
        let text: String = row.try_get("text").map_err(db::sql_error)?;
        let raw_locator: String = row.try_get("locator").map_err(db::sql_error)?;
        let raw_metadata: String = row.try_get("metadata").map_err(db::sql_error)?;
        let locator: Value = serde_json::from_str(&raw_locator).map_err(db::json_error)?;
        let metadata: Value = serde_json::from_str(&raw_metadata).map_err(db::json_error)?;
        let key = (chunk_role(&metadata), kind, source_revision, text);
        grouped.entry(key).or_default().push_back(ExistingChunk {
            id,
            locator,
            metadata,
        });
    }
    Ok(grouped)
}

fn take_matching(
    grouped: &mut HashMap<IdentityKey, VecDeque<ExistingChunk>>,
    key: &IdentityKey,
    locator: &Value,
) -> Option<ExistingChunk> {
    let bucket = grouped.get_mut(key)?;
    let position = bucket.iter().position(|row| row.locator == *locator)?;
    bucket.remove(position)
}

async fn delete_by_ids(
    tx: &mut Transaction<'_, Sqlite>,
    (space_id, node_id): (&str, &str),
    ids: &[String],
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    grounding::references::detach_chunks(tx, space_id, ids).await?;
    sqlx::query(
        "DELETE FROM spaces_knowledge_chunks \
         WHERE space_id=$1 AND node_id=$2 AND id IN (SELECT value FROM json_each($3))",
    )
    .bind(space_id)
    .bind(node_id)
    .bind(sqlx::types::Json(ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

fn chunk_role(metadata: &Value) -> Option<String> {
    metadata
        .get("role")
        .and_then(Value::as_str)
        .map(str::to_owned)
}
