use arrow_array::RecordBatch;
use futures::TryStreamExt;
use lancedb::{
    query::{ExecutableQuery, QueryBase, Select},
    Table,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};

use crate::{
    ai::{
        capabilities::embedding,
        contracts::embedding_wire::{EmbeddingExecutionIdentity, EmbeddingRole},
    },
    spaces::db,
};

use super::{
    current_chunks, generation,
    lancedb_store::{connect_retrieval_db, open_knowledge_table_if_exists},
    predicates::escape_sql_literal,
    schema::VECTOR_COLUMN,
    search_results::batches_to_values,
    validation::validate_query_vector,
};

pub(crate) async fn embed_query<R: Runtime>(
    app: &AppHandle<R>,
    query: &str,
    owner_id: &str,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Vec<f32>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("Semantic search query is required".to_owned());
    }
    let resources = crate::local_resources::preparation::acquire_verified_models(
        app,
        crate::local_resources::models::catalog::TEXT_EMBEDDING_MODELS,
        cancellation,
    )
    .await?;
    let identity = EmbeddingExecutionIdentity {
        owner_kind: "knowledge_query".to_owned(),
        owner_id: owner_id.to_owned(),
        attempt_id: uuid::Uuid::new_v4().to_string(),
        attempt_epoch: 1,
        input_revision: format!("sha256:{:x}", Sha256::digest(query.as_bytes())),
        deadline_ms: deadline_after(30_000)?,
    };
    let response = embedding::embed(
        app,
        vec![query.to_owned()],
        EmbeddingRole::Query,
        &identity,
        &resources,
        cancellation,
    )
    .await?;
    response
        .vectors
        .into_iter()
        .next()
        .ok_or_else(|| "Embedding provider returned no query vector".to_owned())
}

fn deadline_after(duration_ms: i64) -> Result<u64, String> {
    crate::clock::unix_time_millis()?
        .checked_add(duration_ms)
        .and_then(|deadline| u64::try_from(deadline).ok())
        .ok_or_else(|| "Embedding deadline exceeds the supported range".to_owned())
}

pub(crate) async fn search_semantic_vector<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    query: &[f32],
    node_ids: Option<&[String]>,
    exclude_node_id: Option<&str>,
    limit: i64,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(app).await?;
    let generation = generation::active(pool, space_id, embedding::EMBEDDING_SPEC_ID).await?;
    let index = connect_retrieval_db(app).await?;
    let Some(table) = open_knowledge_table_if_exists(&index).await? else {
        return Err(generation::REBUILD_REQUIRED.to_owned());
    };
    let filter = build_filter(
        space_id,
        &generation.id,
        &generation.embedding_spec_id,
        node_ids,
        exclude_node_id,
    );
    let values = query_vectors(&table, query, filter, limit).await?;
    let still_active = generation::active(pool, space_id, embedding::EMBEDDING_SPEC_ID).await?;
    if still_active.id != generation.id {
        return Err(generation::REBUILD_REQUIRED.to_owned());
    }
    current_chunks::filter(pool, &[(space_id, &generation)], values).await
}

pub(super) async fn query_vectors(
    table: &Table,
    query: &[f32],
    filter: String,
    limit: i64,
) -> Result<Vec<Value>, String> {
    validate_query_vector(query, limit)?;
    let batches = table
        .vector_search(query.to_vec())
        .map_err(|error| format!("Failed to prepare LanceDB vector search: {error}"))?
        .column(VECTOR_COLUMN)
        .only_if(filter)
        .select(Select::columns(&[
            "space_id",
            "chunk_id",
            "node_id",
            "node_kind",
            "node_title",
            "chunk_kind",
            "source_revision",
            "locator",
            "text",
            "metadata",
            "_distance",
        ]))
        .limit(limit as usize)
        .execute()
        .await
        .map_err(|error| format!("Failed to execute LanceDB vector search: {error}"))?
        .try_collect::<Vec<RecordBatch>>()
        .await
        .map_err(|error| format!("Failed to read LanceDB vector search results: {error}"))?;
    batches_to_values(&batches)
}

fn build_filter(
    space_id: &str,
    generation_id: &str,
    embedding_spec_id: &str,
    node_ids: Option<&[String]>,
    exclude_node_id: Option<&str>,
) -> String {
    let mut filter = format!(
        "space_id = '{}' AND generation_id = '{}' AND embedding_spec_id = '{}'",
        escape_sql_literal(space_id),
        escape_sql_literal(generation_id),
        escape_sql_literal(embedding_spec_id)
    );
    if let Some(ids) = node_ids.filter(|ids| !ids.is_empty()) {
        let quoted_ids = ids
            .iter()
            .map(|id| format!("'{}'", escape_sql_literal(id)))
            .collect::<Vec<_>>()
            .join(",");
        filter.push_str(" AND node_id IN (");
        filter.push_str(&quoted_ids);
        filter.push(')');
    }
    if let Some(id) = exclude_node_id {
        filter.push_str(" AND node_id != '");
        filter.push_str(&escape_sql_literal(id));
        filter.push('\'');
    }
    filter
}
