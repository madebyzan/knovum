use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use crate::{
    ai::{
        capabilities::embedding,
        contracts::embedding_wire::{EmbeddingExecutionIdentity, EmbeddingRole},
    },
    local_resources::preparation,
    spaces::db,
};

use super::{
    generation::{self, IndexGeneration},
    index_writer::{upsert_embeddings, KnowledgeEmbeddingInput},
};

struct KnowledgeChunkSource {
    id: String,
    source_revision: String,
    text: String,
}

/// One batch is at most [`embedding::active_text_provider`]'s batch size of short chunks through a
/// loaded model. The budget covers that encode alone: the slot wait and the worker's cold start
/// are bounded separately, so this number no longer has to absorb them.
const BATCH_ENCODE_BUDGET_MS: u64 = 120_000;

pub(super) async fn run<R: Runtime>(
    app: AppHandle<R>,
    space_id: &str,
    node_ids: Option<Vec<String>>,
    resources: preparation::PreparedModels,
    generation: IndexGeneration,
    activate: bool,
    cancellation: &CancellationToken,
) -> Result<usize, String> {
    let pool = db::pool(&app).await?;
    let mut indexed = 0usize;
    let started = std::time::Instant::now();
    log::info!("knowledge_index space={} event=started", space_id);
    loop {
        if cancellation.is_cancelled() {
            return Err("knowledge-indexing-cancelled".into());
        }
        let chunks = load_unindexed_chunks(
            pool,
            space_id,
            &generation,
            node_ids.as_ref(),
            embedding::active_text_provider().max_batch_size() as i64,
        )
        .await?;
        if chunks.is_empty() {
            if activate {
                generation::activate(pool, space_id, &generation).await?;
                super::retired_generations::notify_pending();
            }
            log::info!(
                "knowledge_index space={} event=completed indexed={} elapsed_ms={}",
                space_id,
                indexed,
                started.elapsed().as_millis()
            );
            return Ok(indexed);
        }
        let input_revision = batch_revision(&chunks);
        let texts = chunks.iter().map(|chunk| chunk.text.clone()).collect();
        let identity = EmbeddingExecutionIdentity {
            owner_kind: "knowledge_index".to_owned(),
            owner_id: space_id.to_owned(),
            attempt_id: generation.id.clone(),
            attempt_epoch: 1,
            input_revision,
            budget_ms: BATCH_ENCODE_BUDGET_MS,
        };
        let response = embedding::embed(
            &app,
            texts,
            EmbeddingRole::Document,
            &identity,
            &resources,
            cancellation,
        )
        .await
        .map_err(|error| error.to_string())?;
        if response.vectors.len() != chunks.len() {
            return Err("Embedding provider returned an incomplete rebuild batch".to_owned());
        }
        let inputs = chunks
            .into_iter()
            .zip(response.vectors)
            .map(|(chunk, vector)| KnowledgeEmbeddingInput {
                generation_id: generation.id.clone(),
                embedding_spec_id: generation.embedding_spec_id.clone(),
                chunk_id: chunk.id,
                source_revision: chunk.source_revision,
                dimensions: embedding::active_text_provider().dimensions() as i64,
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
    }
}

fn batch_revision(chunks: &[KnowledgeChunkSource]) -> String {
    let mut digest = Sha256::new();
    for chunk in chunks {
        digest.update(chunk.id.as_bytes());
        digest.update([0]);
        digest.update(chunk.source_revision.as_bytes());
        digest.update([0]);
    }
    format!("sha256:{}", hex::encode(digest.finalize()))
}

async fn load_unindexed_chunks(
    pool: &SqlitePool,
    space_id: &str,
    generation: &IndexGeneration,
    node_ids: Option<&Vec<String>>,
    limit: i64,
) -> Result<Vec<KnowledgeChunkSource>, String> {
    let rows = sqlx::query(
        "SELECT c.id,c.source_revision,c.text FROM spaces_current_knowledge_chunks c \
         LEFT JOIN spaces_chunk_embeddings e ON e.chunk_id=c.id AND e.generation_id=$2 \
           AND e.embedding_spec_id=$3 AND e.source_revision=c.source_revision \
         WHERE c.space_id=$1 AND e.chunk_id IS NULL AND trim(c.text)<>'' \
           AND ($5 IS NULL OR c.node_id IN (SELECT value FROM json_each($5))) \
         ORDER BY c.updated_at,c.id LIMIT $4",
    )
    .bind(space_id)
    .bind(&generation.id)
    .bind(&generation.embedding_spec_id)
    .bind(limit)
    .bind(node_ids.map(sqlx::types::Json))
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
