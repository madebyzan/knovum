use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

use super::schema::IndexedKnowledgeChunk;

#[derive(Debug)]
pub(super) struct EmbeddingSource {
    pub generation_id: String,
    pub embedding_spec_id: String,
    pub chunk_id: String,
    pub source_revision: String,
    pub dimensions: i64,
    pub vector: Vec<f32>,
}

/// Loads the persisted chunk metadata for each embedding input and combines it with the
/// caller-provided vector under the publication write transaction. Revision mismatch
/// is an explicit stale-job failure.
pub(super) async fn load_records(
    tx: &mut Transaction<'_, Sqlite>,
    embeddings: Vec<EmbeddingSource>,
) -> Result<Vec<IndexedKnowledgeChunk>, String> {
    if embeddings.is_empty() {
        return Ok(Vec::new());
    }
    let now = crate::clock::unix_time_millis()?;

    let chunk_ids: Vec<&str> = embeddings.iter().map(|e| e.chunk_id.as_str()).collect();
    let chunk_ids_json = serde_json::to_string(&chunk_ids).map_err(db::json_error)?;
    let rows = sqlx::query(
        "SELECT c.id AS chunk_id, c.space_id, c.node_id, n.kind AS node_kind, \
         json_extract(n.data, '$.title') AS node_title, c.kind AS chunk_kind, \
         c.source_revision, c.text, c.locator, c.metadata \
         FROM spaces_current_knowledge_chunks c \
         JOIN spaces_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.id IN (SELECT value FROM json_each($1)) AND n.deleted_at IS NULL",
    )
    .bind(chunk_ids_json)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;

    let mut by_chunk_id = std::collections::HashMap::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("chunk_id").map_err(db::sql_error)?;
        by_chunk_id.insert(id, row);
    }

    let mut records = Vec::with_capacity(embeddings.len());
    for embedding in embeddings {
        let row = by_chunk_id.get(&embedding.chunk_id);
        let row = match row {
            Some(row) => {
                let rev: String = row.try_get("source_revision").map_err(db::sql_error)?;
                if rev != embedding.source_revision {
                    None
                } else {
                    Some(row)
                }
            }
            None => None,
        };
        let row = row.ok_or_else(|| {
            format!(
                "Knowledge chunk {} no longer has source revision {}",
                embedding.chunk_id, embedding.source_revision
            )
        })?;
        records.push(IndexedKnowledgeChunk {
            generation_id: embedding.generation_id,
            embedding_spec_id: embedding.embedding_spec_id,
            chunk_id: row.try_get("chunk_id").map_err(db::sql_error)?,
            space_id: row.try_get("space_id").map_err(db::sql_error)?,
            node_id: row.try_get("node_id").map_err(db::sql_error)?,
            node_kind: row.try_get("node_kind").map_err(db::sql_error)?,
            node_title: row
                .try_get::<Option<String>, _>("node_title")
                .map_err(db::sql_error)?
                .ok_or_else(|| "Knowledge node is missing its required title".to_owned())?,
            chunk_kind: row.try_get("chunk_kind").map_err(db::sql_error)?,
            source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
            dimensions: embedding.dimensions as i32,
            text: row.try_get("text").map_err(db::sql_error)?,
            locator: row.try_get("locator").map_err(db::sql_error)?,
            metadata: row.try_get("metadata").map_err(db::sql_error)?,
            vector: embedding.vector,
            updated_at: now,
        });
    }
    Ok(records)
}

/// Persists a marker row per successfully indexed chunk. The marker is used by
/// `list_missing_embedding_chunks` — the vector itself lives only in LanceDB.
pub(super) async fn write_index_markers(
    tx: &mut Transaction<'_, Sqlite>,
    records: &[IndexedKnowledgeChunk],
) -> Result<(), String> {
    for record in records {
        sqlx::query(
            "INSERT INTO spaces_chunk_embeddings(\
               generation_id,chunk_id,source_revision,embedding_spec_id,dimensions,updated_at\
             ) VALUES($1,$2,$3,$4,$5,$6) \
             ON CONFLICT(generation_id,chunk_id) DO UPDATE SET \
               source_revision=excluded.source_revision, \
               embedding_spec_id=excluded.embedding_spec_id, \
               dimensions=excluded.dimensions, updated_at=excluded.updated_at",
        )
        .bind(&record.generation_id)
        .bind(&record.chunk_id)
        .bind(&record.source_revision)
        .bind(&record.embedding_spec_id)
        .bind(record.dimensions)
        .bind(record.updated_at)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}
