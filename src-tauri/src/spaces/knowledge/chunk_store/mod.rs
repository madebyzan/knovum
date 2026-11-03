mod revisions;
mod validation;
mod write;

use sqlx::{Sqlite, Transaction};

use write::{expected_revision, insert};

use super::{grounding, indexing::KnowledgeChunkInput};

use crate::spaces::db;

pub(in crate::spaces) use revisions::{load_revision_contracts, NodeChunkRevisions};
use validation::validate_origin;
pub(in crate::spaces) use validation::validate_values;

pub async fn replace(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    origin: &str,
    chunks: &[KnowledgeChunkInput],
    now: i64,
) -> Result<(), String> {
    validate_origin(origin)?;
    if chunks.is_empty() {
        return clear(tx, space_id, node_id, origin).await;
    }
    let expected_revision = expected_revision(tx, space_id, node_id, origin).await?;
    replace_with_expected(
        tx,
        space_id,
        node_id,
        origin,
        chunks,
        &expected_revision,
        now,
    )
    .await
}

/// Replaces the passages one writer owns inside an origin it shares, identified by their metadata
/// role; passages other writers put in that origin keep their identities, and so their citations.
pub(in crate::spaces) async fn replace_role(
    tx: &mut Transaction<'_, Sqlite>,
    (space_id, node_id): (&str, &str),
    (origin, role): (&str, &str),
    chunks: &[KnowledgeChunkInput],
    now: i64,
) -> Result<(), String> {
    validate_origin(origin)?;
    if chunks.iter().any(|chunk| {
        chunk
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("role"))
            .and_then(serde_json::Value::as_str)
            != Some(role)
    }) {
        return Err(format!("Every replaced passage must carry the {role} role"));
    }
    let expected_revision = expected_revision(tx, space_id, node_id, origin).await?;
    validate_chunks(origin, chunks, &expected_revision)?;
    delete_chunks(tx, (space_id, node_id), origin, Some(role)).await?;
    for chunk in chunks {
        insert(tx, space_id, node_id, origin, chunk, now).await?;
    }
    Ok(())
}

fn validate_chunks(
    origin: &str,
    chunks: &[KnowledgeChunkInput],
    expected_revision: &str,
) -> Result<(), String> {
    for chunk in chunks {
        validate_values(
            origin,
            &chunk.kind,
            &chunk.text,
            &chunk.locator,
            chunk.metadata.as_ref(),
            expected_revision,
        )?;
    }
    Ok(())
}

pub(crate) async fn clear(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    origin: &str,
) -> Result<(), String> {
    validate_origin(origin)?;
    delete_chunks(tx, (space_id, node_id), origin, None).await
}

/// Deletes an origin's passages, or only those of one metadata role, detaching their citations.
async fn delete_chunks(
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
    grounding::references::detach_chunks(tx, space_id, &old_ids).await?;
    sqlx::query(
        "DELETE FROM spaces_knowledge_chunks WHERE node_id=$1 AND space_id=$2 AND origin=$3 \
         AND ($4 IS NULL OR json_extract(metadata,'$.role')=$4)",
    )
    .bind(node_id)
    .bind(space_id)
    .bind(origin)
    .bind(role)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(crate) async fn detach_canonical(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
) -> Result<(), String> {
    grounding::references::detach_chunks(tx, space_id, &[format!("node:{node_id}")]).await
}

pub(crate) async fn replace_with_expected(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    origin: &str,
    chunks: &[KnowledgeChunkInput],
    expected_revision: &str,
    now: i64,
) -> Result<(), String> {
    replace_internal(
        tx,
        ReplaceInput {
            space_id,
            node_id,
            origin,
            chunks,
            expected_revision,
            now,
        },
    )
    .await
}

struct ReplaceInput<'a> {
    space_id: &'a str,
    node_id: &'a str,
    origin: &'a str,
    chunks: &'a [KnowledgeChunkInput],
    expected_revision: &'a str,
    now: i64,
}

async fn replace_internal(
    tx: &mut Transaction<'_, Sqlite>,
    input: ReplaceInput<'_>,
) -> Result<(), String> {
    let ReplaceInput {
        space_id,
        node_id,
        origin,
        chunks,
        expected_revision,
        now,
    } = input;
    validate_origin(origin)?;
    validate_chunks(origin, chunks, expected_revision)?;
    delete_chunks(tx, (space_id, node_id), origin, None).await?;
    for chunk in chunks {
        insert(tx, space_id, node_id, origin, chunk, now).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
