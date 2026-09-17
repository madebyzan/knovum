use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};
use uuid::Uuid;

use super::super::indexing::KnowledgeChunkInput;
use crate::spaces::db;

pub(super) async fn expected_revision(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    origin: &str,
) -> Result<String, String> {
    let row = sqlx::query(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Knowledge chunk node is unavailable in this Space".to_string())?;
    let raw: String = row.try_get("data").map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    let key = match origin {
        "derived" => "knowledgeRevision",
        "source" | "media" => "sourceRevision",
        _ => return Err("Only supplemental knowledge origins can be replaced explicitly".into()),
    };
    data.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("Knowledge {origin} chunks require node {key}"))
}

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    origin: &str,
    chunk: &KnowledgeChunkInput,
    now: i64,
) -> Result<(), String> {
    let source_revision = chunk
        .locator
        .get("sourceRevision")
        .and_then(Value::as_str)
        .ok_or_else(|| "Knowledge chunk locator is missing sourceRevision".to_owned())?;
    let locator = serde_json::to_string(&chunk.locator).map_err(db::json_error)?;
    let empty_metadata = Value::Object(Default::default());
    let metadata = serde_json::to_string(chunk.metadata.as_ref().unwrap_or(&empty_metadata))
        .map_err(db::json_error)?;
    sqlx::query(
        r#"INSERT INTO spaces_knowledge_chunks(
             id, space_id, node_id, origin, kind, source_revision, text, locator, metadata,
             created_at, updated_at
           ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$10)"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(space_id)
    .bind(node_id)
    .bind(origin)
    .bind(&chunk.kind)
    .bind(source_revision)
    .bind(chunk.text.trim())
    .bind(locator)
    .bind(metadata)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
