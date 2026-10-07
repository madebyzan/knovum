use arrow_array::RecordBatchIterator;
use lancedb::Connection;
use serde::Deserialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

use super::{
    arrow_batch::records_to_batch,
    generation::{self, IndexGeneration},
    invalidation_outbox,
    lancedb_store::connect_retrieval_db,
    record_loader::{load_records, write_index_markers, EmbeddingSource},
    schema::TABLE_KNOWLEDGE_CHUNKS,
};

const MAX_EMBEDDING_BATCH: usize = 64;

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
) -> Result<usize, String> {
    let pool = db::pool(&app).await?;
    let index = connect_retrieval_db(&app).await?;
    publish_embeddings(pool, &index, embeddings).await
}

pub(super) async fn publish_embeddings(
    pool: &SqlitePool,
    index: &Connection,
    embeddings: Vec<KnowledgeEmbeddingInput>,
) -> Result<usize, String> {
    validate_batch(&embeddings)?;
    // Serialize current-chunk validation, pending invalidations and index publication.
    // Model inference has already completed before this short write boundary begins.
    let mut tx = db::begin_write(pool).await?;
    let generation = generation_from_batch(&embeddings)?;
    generation::require_writable(&mut tx, &generation).await?;
    let records = load_records(&mut tx, embeddings.into_iter().map(into_source).collect()).await?;
    let spaces: std::collections::BTreeSet<_> = records
        .iter()
        .map(|record| record.space_id.as_str())
        .collect();
    for space_id in spaces {
        invalidation_outbox::drain_pending(index, &mut tx, Some(space_id)).await?;
    }
    let written = records.len();
    let dimensions = records[0].dimensions;
    let batch = records_to_batch(&records, dimensions)?;
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
        upsert
            .execute(Box::new(data))
            .await
            .map_err(|error| format!("Failed to upsert LanceDB knowledge index rows: {error}"))?;
    } else {
        index
            .create_table(TABLE_KNOWLEDGE_CHUNKS, batch)
            .execute()
            .await
            .map_err(|error| format!("Failed to create LanceDB knowledge table: {error}"))?;
    }
    write_index_markers(&mut tx, &records).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(written)
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
