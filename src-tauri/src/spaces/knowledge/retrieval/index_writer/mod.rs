use std::fmt;

use arrow_array::RecordBatchIterator;
use lancedb::Connection;
use serde::Deserialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

use super::{
    arrow_batch::records_to_batch,
    chunk_loader::{load_indexed_chunks, write_index_markers, EmbeddingSource},
    generation::{self, IndexGeneration},
    invalidation_outbox,
    lancedb_store::connect_retrieval_db,
    schema::TABLE_KNOWLEDGE_CHUNKS,
};

const MAX_EMBEDDING_BATCH: usize = 64;

#[derive(Debug)]
pub(crate) enum PublishError {
    StaleRevision(String),
    Failed(String),
}

impl fmt::Display for PublishError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleRevision(message) | Self::Failed(message) => formatter.write_str(message),
        }
    }
}

impl From<String> for PublishError {
    fn from(message: String) -> Self {
        Self::Failed(message)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KnowledgeEmbeddingInput {
    pub generation_id: String,
    pub embedding_spec_id: String,
    pub chunk_id: String,
    pub source_revision: String,
    pub dimensions: i64,
    pub vector: Vec<f64>,
}

pub(crate) async fn upsert_embeddings<R: Runtime>(
    app: AppHandle<R>,
    embeddings: Vec<KnowledgeEmbeddingInput>,
) -> Result<usize, PublishError> {
    let pool = db::pool(&app).await?;
    let index = connect_retrieval_db(&app).await?;
    publish_embeddings(pool, &index, embeddings).await
}

pub(super) async fn publish_embeddings(
    pool: &SqlitePool,
    index: &Connection,
    embeddings: Vec<KnowledgeEmbeddingInput>,
) -> Result<usize, PublishError> {
    validate_batch(&embeddings)?;
    let generation = generation_from_batch(&embeddings)?;
    // Phase 1: short SQLite read to validate generation and load current chunk metadata.
    let chunks = {
        let mut read_tx = pool.begin().await.map_err(db::sql_error)?;
        generation::require_writable(&mut read_tx, &generation).await?;
        load_indexed_chunks(
            &mut read_tx,
            embeddings.into_iter().map(into_source).collect(),
        )
        .await?
    };
    let spaces: std::collections::BTreeSet<String> =
        chunks.iter().map(|chunk| chunk.space_id.clone()).collect();
    for space_id in &spaces {
        invalidation_outbox::drain_space_with_connection(index, pool, space_id).await?;
    }
    // Phase 2: write vectors to LanceDB under process-local write lock without holding SQLite.
    let written = chunks.len();
    let dimensions = chunks[0].dimensions;
    let batch = records_to_batch(&chunks, dimensions)?;
    {
        let _lance_guard = invalidation_outbox::acquire_lance_write_lock().await;
        let existing = index
            .table_names()
            .execute()
            .await
            .map_err(|error| format!("Failed to inspect LanceDB retrieval tables: {error}"))?;
        if existing.iter().any(|name| name == TABLE_KNOWLEDGE_CHUNKS) {
            let table = index
                .open_table(TABLE_KNOWLEDGE_CHUNKS)
                .execute()
                .await
                .map_err(|error| format!("Failed to open LanceDB knowledge index: {error}"))?;
            let schema = batch.schema();
            let data = RecordBatchIterator::new(vec![Ok(batch)], schema);
            let mut upsert = table.merge_insert(&["generation_id", "chunk_id"]);
            upsert
                .when_matched_update_all(None)
                .when_not_matched_insert_all();
            upsert.execute(Box::new(data)).await.map_err(|error| {
                format!("Failed to upsert LanceDB knowledge index rows: {error}")
            })?;
        } else {
            index
                .create_table(TABLE_KNOWLEDGE_CHUNKS, batch)
                .execute()
                .await
                .map_err(|error| format!("Failed to create LanceDB knowledge table: {error}"))?;
        }
    }
    // Phase 3: short SQLite write transaction to verify revisions still match and record markers.
    let mut tx = db::begin_write(pool).await?;
    generation::require_writable(&mut tx, &generation).await?;
    if let Err(stale) = verify_chunks_still_current(&mut tx, &chunks).await {
        let now = crate::foundation::clock::unix_time_millis()?;
        let mut nodes_by_space: std::collections::BTreeMap<&str, Vec<String>> =
            std::collections::BTreeMap::new();
        for chunk in &chunks {
            nodes_by_space
                .entry(chunk.space_id.as_str())
                .or_default()
                .push(chunk.node_id.clone());
        }
        for (space_id, node_ids) in nodes_by_space {
            invalidation_outbox::enqueue_nodes(&mut tx, space_id, &node_ids, now).await?;
        }
        tx.commit().await.map_err(db::sql_error)?;
        return Err(stale);
    }
    write_index_markers(&mut tx, &chunks).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(written)
}

async fn verify_chunks_still_current(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    chunks: &[super::schema::IndexedKnowledgeChunk],
) -> Result<(), PublishError> {
    let chunk_ids: Vec<&str> = chunks.iter().map(|chunk| chunk.chunk_id.as_str()).collect();
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT c.id, c.source_revision \
         FROM spaces_current_knowledge_chunks c \
         JOIN spaces_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.id IN (SELECT value FROM json_each($1)) AND n.deleted_at IS NULL",
    )
    .bind(sqlx::types::Json(&chunk_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let revisions: std::collections::HashMap<String, String> = rows.into_iter().collect();
    for chunk in chunks {
        if revisions.get(&chunk.chunk_id).map(String::as_str)
            != Some(chunk.source_revision.as_str())
        {
            return Err(PublishError::StaleRevision(format!(
                "Knowledge chunk {} changed while publishing embeddings",
                chunk.chunk_id
            )));
        }
    }
    Ok(())
}

fn into_source(input: KnowledgeEmbeddingInput) -> EmbeddingSource {
    EmbeddingSource {
        generation_id: input.generation_id,
        embedding_spec_id: input.embedding_spec_id,
        chunk_id: input.chunk_id,
        source_revision: input.source_revision,
        dimensions: input.dimensions,
        vector: input.vector.into_iter().map(|value| value as f32).collect(),
    }
}

fn validate_batch(embeddings: &[KnowledgeEmbeddingInput]) -> Result<(), String> {
    if embeddings.is_empty() || embeddings.len() > MAX_EMBEDDING_BATCH {
        return Err(format!(
            "Embedding writes require between 1 and {MAX_EMBEDDING_BATCH} rows"
        ));
    }
    let mut chunk_ids = std::collections::HashSet::with_capacity(embeddings.len());
    for embedding in embeddings {
        validate_embedding(embedding)?;
        if !chunk_ids.insert(embedding.chunk_id.as_str()) {
            return Err("Embedding write batch contains a duplicate chunk ID".into());
        }
    }
    Ok(())
}

fn validate_embedding(embedding: &KnowledgeEmbeddingInput) -> Result<(), String> {
    if embedding.chunk_id.is_empty()
        || embedding.generation_id.is_empty()
        || embedding.embedding_spec_id.is_empty()
        || embedding.source_revision.is_empty()
    {
        return Err("Embedding generation, spec, chunk and revision are required".into());
    }
    if embedding.dimensions <= 0 || embedding.dimensions != embedding.vector.len() as i64 {
        return Err("Embedding dimensions do not match the vector length".into());
    }
    if embedding.vector.iter().any(|value| !value.is_finite()) {
        return Err("Embedding vectors must contain only finite numbers".into());
    }
    Ok(())
}

fn generation_from_batch(
    embeddings: &[KnowledgeEmbeddingInput],
) -> Result<IndexGeneration, String> {
    let first = &embeddings[0];
    if embeddings.iter().any(|embedding| {
        embedding.generation_id != first.generation_id
            || embedding.embedding_spec_id != first.embedding_spec_id
    }) {
        return Err("Embedding write batch contains mixed index generations".to_owned());
    }
    Ok(IndexGeneration {
        id: first.generation_id.clone(),
        embedding_spec_id: first.embedding_spec_id.clone(),
    })
}

#[cfg(test)]
pub(super) mod tests;
