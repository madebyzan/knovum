use lancedb::Table;
use serde::Deserialize;
use tauri::{AppHandle, Runtime};

use super::{
    arrow_batch::records_to_batch,
    lancedb_store::{connect_retrieval_db, open_knowledge_table_if_exists},
    predicates::{
        build_delete_chunk_predicate, build_delete_node_predicate, build_delete_space_predicate,
    },
    record_loader::{load_records, write_index_markers, EmbeddingSource},
    schema::{IndexedKnowledgeChunk, TABLE_KNOWLEDGE_CHUNKS},
};

const MAX_EMBEDDING_BATCH: usize = 64;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KnowledgeEmbeddingInput {
    pub chunk_id: String,
    pub source_revision: String,
    pub model: String,
    pub dimensions: i64,
    pub vector: Vec<f64>,
}

pub(crate) async fn upsert_embeddings<R: Runtime>(
    app: AppHandle<R>,
    embeddings: Vec<KnowledgeEmbeddingInput>,
) -> Result<usize, String> {
    validate_batch(&embeddings)?;
    let records = load_records(&app, embeddings.into_iter().map(into_source).collect()).await?;
    if records.is_empty() {
        return Ok(0);
    }
    let written = records.len();
    let dimensions = records[0].dimensions;
    let batch = records_to_batch(&records, dimensions)?;
    let db = connect_retrieval_db(&app).await?;
    let existing = db
        .table_names()
        .execute()
        .await
        .map_err(|error| format!("Failed to inspect LanceDB retrieval tables: {error}"))?;
    if existing.iter().any(|name| name == TABLE_KNOWLEDGE_CHUNKS) {
        let table = db
            .open_table(TABLE_KNOWLEDGE_CHUNKS)
            .execute()
            .await
            .map_err(|error| format!("Failed to open LanceDB knowledge index: {error}"))?;
        delete_existing_chunks(&table, &records).await?;
        table
            .add(batch)
            .execute()
            .await
            .map_err(|error| format!("Failed to append LanceDB knowledge index rows: {error}"))?;
    } else {
        db.create_table(TABLE_KNOWLEDGE_CHUNKS, batch)
            .execute()
            .await
            .map_err(|error| format!("Failed to create LanceDB knowledge table: {error}"))?;
    }
    write_index_markers(&app, &records).await?;
    Ok(written)
}

pub(super) async fn delete_node_embeddings<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    node_ids: &[String],
) -> Result<(), String> {
    if node_ids.is_empty() {
        return Ok(());
    }
    let Some(table) = open_knowledge_table_if_exists(app).await? else {
        // Retrieval index has never been created — nothing to delete.
        return Ok(());
    };
    let predicate = build_delete_node_predicate(space_id, node_ids);
    table
        .delete(predicate.as_str())
        .await
        .map_err(|error| format!("Failed to delete LanceDB node index rows: {error}"))?;
    Ok(())
}

pub(super) async fn delete_space_embeddings<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
) -> Result<(), String> {
    let Some(table) = open_knowledge_table_if_exists(app).await? else {
        return Ok(());
    };
    let predicate = build_delete_space_predicate(space_id);
    table
        .delete(predicate.as_str())
        .await
        .map_err(|error| format!("Failed to delete LanceDB space index rows: {error}"))?;
    Ok(())
}

async fn delete_existing_chunks(
    table: &Table,
    records: &[IndexedKnowledgeChunk],
) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }
    table
        .delete(build_delete_chunk_predicate(records).as_str())
        .await
        .map_err(|error| format!("Failed to delete stale LanceDB index rows: {error}"))?;
    Ok(())
}

fn into_source(input: KnowledgeEmbeddingInput) -> EmbeddingSource {
    EmbeddingSource {
        chunk_id: input.chunk_id,
        source_revision: input.source_revision,
        model: input.model,
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
        || embedding.source_revision.is_empty()
        || embedding.model.is_empty()
    {
        return Err("Embedding chunk, revision and model are required".into());
    }
    if embedding.dimensions <= 0 || embedding.dimensions != embedding.vector.len() as i64 {
        return Err("Embedding dimensions do not match the vector length".into());
    }
    if embedding.vector.iter().any(|value| !value.is_finite()) {
        return Err("Embedding vectors must contain only finite numbers".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
