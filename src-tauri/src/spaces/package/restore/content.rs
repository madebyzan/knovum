use sqlx::{Sqlite, Transaction};

use super::contracts::{PackageChunkRow, RestorePackageRequest};
use crate::spaces::knowledge::chunk_store;
use crate::spaces::{db, knowledge::grounding};

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
        let seeds = generated
            .into_iter()
            .map(
                |chunk| crate::spaces::knowledge::indexing::KnowledgeChunkInput {
                    kind: chunk.kind.to_owned(),
                    text: chunk.text,
                    locator: chunk.locator,
                    metadata: (!chunk.metadata.is_null()).then_some(chunk.metadata),
                },
            )
            .collect::<Vec<_>>();
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
    row: &PackageChunkRow,
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

pub(super) async fn insert_threads(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.threads {
        grounding::validate_title(&row.title)?;
        grounding::validate_scope_nodes(tx, &request.space.id, &row.scope, &row.scope_node_ids)
            .await?;
        let ids = serde_json::to_string(&row.scope_node_ids).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_ai_threads(id,space_id,scope,scope_node_ids,title,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.scope).bind(ids).bind(&row.title)
            .bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn insert_messages(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.messages {
        grounding::validate_role(&row.role)?;
        grounding::validate_message_metadata(
            &row.content,
            row.provider.as_deref(),
            row.model.as_deref(),
        )?;
        let contract = grounding::thread_contract(tx, &row.thread_id).await?;
        if contract.space_id != request.space.id {
            return Err("Package grounded message thread belongs to another Space".into());
        }
        grounding::validate_historical_citations(tx, &contract, &row.citations).await?;
        let citations = serde_json::to_string(&row.citations).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_ai_messages(id,thread_id,role,content,citations,provider,model,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)",
        ).bind(&row.id).bind(&row.thread_id).bind(&row.role).bind(&row.content).bind(citations)
            .bind(row.provider.as_deref()).bind(row.model.as_deref()).bind(row.created_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}
