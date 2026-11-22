mod revisions;
mod sync;
mod validation;
mod write;

use sqlx::{Sqlite, Transaction};

use write::expected_revision;

use super::{grounding, indexing::KnowledgeChunkInput};

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
    sync::sync_chunks(tx, (space_id, node_id), (origin, Some(role)), chunks, now).await
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
    sync::delete_all(tx, (space_id, node_id), origin, None).await
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
    sync::sync_chunks(tx, (space_id, node_id), (origin, None), chunks, now).await
}

#[cfg(test)]
mod tests;
