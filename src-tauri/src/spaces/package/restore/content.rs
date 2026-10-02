use sqlx::{Sqlite, Transaction};

use super::contracts::{PackageChunk, RestorePackageRequest};
use crate::spaces::db;
use crate::spaces::knowledge::chunk_store;

pub(super) async fn insert_chunks(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    let mut node_ids = request
        .chunks
        .iter()
        .map(|row| row.node_id.clone())
        .collect::<Vec<_>>();
    node_ids.sort();
    node_ids.dedup();
    let revisions = chunk_store::load_revision_contracts(tx, &request.space.id, &node_ids).await?;
    for row in request.chunks.iter().filter(|row| row.origin != "derived") {
        insert_packaged_chunk(tx, request, row, &revisions).await?;
    }
    for node in &request.nodes {
        let packaged = request
            .chunks
            .iter()
            .filter(|row| row.origin == "derived" && row.node_id == node.id)
            .collect::<Vec<_>>();
        let generated = crate::spaces::knowledge::indexing::node::derived_chunks(&node.data)?;
        if !packaged.is_empty() {
            super::derived_chunks::validate_packaged_derived(&packaged, &generated)?;
            for row in packaged {
                insert_packaged_chunk(tx, request, row, &revisions).await?;
            }
            continue;
        }
        if generated.is_empty() {
            continue;
        }
        let revision = node
            .data
            .get("knowledgeRevision")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "Package semantic node is missing knowledgeRevision".to_string())?;
        let seeds = generated.into_iter().map(Into::into).collect::<Vec<_>>();
        chunk_store::replace_with_expected(
            tx,
            &request.space.id,
            &node.id,
            "derived",
            &seeds,
            revision,
            node.updated_at,
        )
        .await?;
    }
    Ok(())
}

async fn insert_packaged_chunk(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
    row: &PackageChunk,
    revisions: &std::collections::HashMap<String, chunk_store::NodeChunkRevisions>,
) -> Result<(), String> {
    if row.origin == "node" {
        return Err("Package must not contain canonical node chunks".into());
    }
    let contract = revisions
        .get(&row.node_id)
        .ok_or_else(|| "Package Knowledge chunk node is unavailable in this Space".to_string())?;
    let expected_revision = contract.expected(&row.origin)?;
    if row.source_revision != expected_revision {
        return Err(
            "Package Knowledge chunk revision does not match the owning node revision".into(),
        );
    }
    chunk_store::validate_values(
        &row.origin,
        &row.kind,
        &row.text,
        &row.locator,
        Some(&row.metadata),
        expected_revision,
    )?;
    let locator = serde_json::to_string(&row.locator).map_err(db::json_error)?;
    let metadata = serde_json::to_string(&row.metadata).map_err(db::json_error)?;
    sqlx::query(
        "INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
    ).bind(&row.id).bind(&request.space.id).bind(&row.node_id).bind(&row.origin).bind(&row.kind)
        .bind(&row.source_revision).bind(&row.text).bind(locator).bind(metadata).bind(row.created_at).bind(row.updated_at)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
