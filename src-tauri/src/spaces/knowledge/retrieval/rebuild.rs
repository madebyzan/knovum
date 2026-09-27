use serde::Serialize;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::{
    ai::capabilities::embedding,
    local_resources::{models::catalog, preparation},
    spaces::db,
};
use tokio_util::sync::CancellationToken;

use super::{
    embedding_contract::{active_batch_size, active_dimensions, active_model_id},
    index_writer::{upsert_embeddings, KnowledgeEmbeddingInput},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RebuildIndexOutput {
    pub rebuilt: bool,
    pub indexed: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexOutput {
    pub indexed: usize,
}

struct KnowledgeChunkSource {
    id: String,
    source_revision: String,
    text: String,
}

pub(crate) async fn rebuild_space_index<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<RebuildIndexOutput, String> {
    if space_id.trim().is_empty() {
        return Err("Knowledge retrieval rebuild requires a space id".to_owned());
    }
    let resources = preparation::acquire_verified_models(
        &app,
        catalog::TEXT_EMBEDDING_MODELS,
        &CancellationToken::new(),
    )
    .await?;
    clear_markers(&app, &space_id).await?;
    let indexed = index_prepared(app, space_id, None, resources, &CancellationToken::new())
        .await?
        .indexed;
    Ok(RebuildIndexOutput {
        rebuilt: true,
        indexed,
    })
}

pub(crate) async fn index_space<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
) -> Result<IndexOutput, String> {
    if space_id.trim().is_empty() {
        return Err("Knowledge retrieval indexing requires a space id".to_owned());
    }
    let resources = preparation::acquire_verified_models(
        &app,
        catalog::TEXT_EMBEDDING_MODELS,
        &CancellationToken::new(),
    )
    .await?;
    index_prepared(
        app,
        space_id,
        node_ids,
        resources,
        &CancellationToken::new(),
    )
    .await
}

pub(crate) async fn prepare_space_index<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    node_ids: Option<Vec<String>>,
    job_id: &str,
    cancellation: &CancellationToken,
) -> Result<IndexOutput, String> {
    if space_id.trim().is_empty() {
        return Err("Knowledge retrieval indexing requires a space id".to_owned());
    }
    let resources = preparation::prepare_models_for_job(
        app,
        preparation::contracts::ResourcePreparationTarget::TextEmbedding,
        &format!("resource:{job_id}"),
        "Preparing semantic search",
        cancellation,
        &mut |_| {},
    )
    .await?;
    index_prepared(
        app.clone(),
        space_id.to_owned(),
        node_ids,
        resources,
        cancellation,
    )
    .await
}

async fn index_prepared<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
    resources: preparation::PreparedModels,
    cancellation: &CancellationToken,
) -> Result<IndexOutput, String> {
    let node_ids_json = node_ids
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(db::json_error)?;
    let mut indexed = 0usize;
    let started = std::time::Instant::now();
    log::info!("knowledge_index space={} event=started", space_id);
    loop {
        if cancellation.is_cancelled() {
            return Err("knowledge-indexing-cancelled".into());
        }
        let chunks = load_unindexed_chunks(
            &app,
            &space_id,
            node_ids_json.as_deref(),
            active_batch_size() as i64,
        )
        .await?;
        if chunks.is_empty() {
            log::info!(
                "knowledge_index space={} event=completed indexed={} elapsed_ms={}",
                space_id,
                indexed,
                started.elapsed().as_millis()
            );
            return Ok(IndexOutput { indexed });
        }
        let texts = chunks
            .iter()
            .map(|chunk| chunk.text.clone())
            .collect::<Vec<_>>();
        let embeddings = embedding::embed(texts, &resources, cancellation)
            .await
            .map_err(|error| error.to_string())?;
        if embeddings.vectors.len() != chunks.len() {
            return Err("Embedding provider returned an incomplete rebuild batch".to_owned());
        }
        let inputs = chunks
            .into_iter()
            .zip(embeddings.vectors)
            .map(|(chunk, vector)| KnowledgeEmbeddingInput {
                chunk_id: chunk.id,
                source_revision: chunk.source_revision,
                model: active_model_id().to_owned(),
                dimensions: active_dimensions() as i64,
                vector: vector.into_iter().map(f64::from).collect(),
            })
            .collect::<Vec<_>>();
        if cancellation.is_cancelled() {
            return Err("knowledge-indexing-cancelled".into());
        }
        let expected = inputs.len();
        let written = upsert_embeddings(app.clone(), inputs).await?;
        if written != expected {
            return Err("Knowledge indexing did not persist its complete batch".to_owned());
        }
        indexed += written;
        log::info!(
            "knowledge_index space={} event=batch_persisted indexed={} elapsed_ms={}",
            space_id,
            indexed,
            started.elapsed().as_millis()
        );
    }
}

async fn clear_markers<R: Runtime>(app: &AppHandle<R>, space_id: &str) -> Result<(), String> {
    let pool = db::pool(app).await?;
    sqlx::query(
        "DELETE FROM spaces_chunk_embeddings WHERE chunk_id IN ( \
         SELECT id FROM spaces_knowledge_chunks WHERE space_id=$1 \
        )",
    )
    .bind(space_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn load_unindexed_chunks<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    node_ids_json: Option<&str>,
    limit: i64,
) -> Result<Vec<KnowledgeChunkSource>, String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query(
        "SELECT c.id,c.source_revision,c.text FROM spaces_current_knowledge_chunks c \
         LEFT JOIN spaces_chunk_embeddings e ON e.chunk_id=c.id AND e.model=$2 AND e.source_revision=c.source_revision \
         WHERE c.space_id=$1 AND e.chunk_id IS NULL AND trim(c.text)<>'' \
           AND ($4 IS NULL OR c.node_id IN (SELECT value FROM json_each($4))) \
         ORDER BY c.updated_at,c.id LIMIT $3",
    )
    .bind(space_id)
    .bind(active_model_id())
    .bind(limit)
    .bind(node_ids_json)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(KnowledgeChunkSource {
                id: row.try_get("id").map_err(db::sql_error)?,
                source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
                text: row.try_get("text").map_err(db::sql_error)?,
            })
        })
        .collect()
}
