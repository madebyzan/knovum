use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};

use crate::{ai::capabilities::embedding, spaces::db};

use super::{generation, indexing_control, rebuild};

/// What a search issued right now would be able to reach in this Space, and how much work stands
/// between that and the Space's whole text.
///
/// This is the reading the index panel shows before it offers to do anything: the same decision
/// `rebuild::index_with_generation` makes — reuse the generation that is answering searches, or
/// start a new one — read out as numbers, so what a person sees is what the next pass will do.
#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeIndexState {
    /// Chunks of this Space that carry text worth indexing.
    pub total: u32,
    /// Chunks the generation the next pass will build on has already encoded.
    pub indexed: u32,
    /// Chunks that are new, edited since they were encoded, or still waiting for a rebuild.
    pub pending: u32,
    /// Whether semantic search can answer from this Space at all. False while the very first index
    /// is being built, after the embedding model changed, and when the vector table was lost.
    pub searchable: bool,
    /// Whether the next pass re-encodes the whole Space rather than filling the gaps.
    pub rebuilding: bool,
    /// Whether a pass holds this Space right now — started here, or by work that needed the index.
    pub indexing: bool,
    /// When the generation that answers searches took over.
    pub indexed_at: Option<i64>,
}

pub(crate) async fn read<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
) -> Result<KnowledgeIndexState, String> {
    if space_id.trim().is_empty() {
        return Err("Knowledge index state requires a space id".to_owned());
    }
    let pool = db::pool(app).await?;
    let materialised = rebuild::index_is_materialised(app).await?;
    let indexing = indexing_control::is_running(space_id)?;
    measure(pool, space_id, materialised, indexing).await
}

pub(super) async fn measure(
    pool: &SqlitePool,
    space_id: &str,
    materialised: bool,
    indexing: bool,
) -> Result<KnowledgeIndexState, String> {
    let spec = embedding::EMBEDDING_SPEC_ID;
    // Markers without a vector table describe an index that cannot answer anything, which is the
    // one case the next pass re-encodes a Space that looks complete.
    let active = match generation::find_active(pool, space_id, spec).await? {
        Some(active) if materialised => Some(active),
        _ => None,
    };
    let measured = match &active {
        Some(active) => Some(active.clone()),
        None => generation::find_building(pool, space_id, spec).await?,
    };
    let total = count_indexable_chunks(pool, space_id).await?;
    let indexed = match &measured {
        Some(generation) => count_encoded_chunks(pool, space_id, generation).await?,
        None => 0,
    };
    Ok(KnowledgeIndexState {
        total: total as u32,
        indexed: indexed.min(total) as u32,
        pending: total.saturating_sub(indexed) as u32,
        searchable: active.is_some(),
        rebuilding: active.is_none() && total > 0,
        indexing,
        indexed_at: active.and_then(|generation| generation.activated_at),
    })
}

async fn count_indexable_chunks(pool: &SqlitePool, space_id: &str) -> Result<i64, String> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_current_knowledge_chunks \
         WHERE space_id=$1 AND trim(text)<>''",
    )
    .bind(space_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)
}

async fn count_encoded_chunks(
    pool: &SqlitePool,
    space_id: &str,
    generation: &generation::IndexGeneration,
) -> Result<i64, String> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_current_knowledge_chunks c \
         JOIN spaces_chunk_embeddings e ON e.chunk_id=c.id AND e.generation_id=$2 \
           AND e.embedding_spec_id=$3 AND e.source_revision=c.source_revision \
         WHERE c.space_id=$1 AND trim(c.text)<>''",
    )
    .bind(space_id)
    .bind(&generation.id)
    .bind(&generation.embedding_spec_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)
}
