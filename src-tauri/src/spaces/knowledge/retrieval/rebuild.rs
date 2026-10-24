use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::{
    ai::capabilities::embedding,
    local_resources::{models::catalog, preparation},
    spaces::db,
};
use tokio_util::sync::CancellationToken;

use super::{generation, generation_build, indexing_control, lancedb_store};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexOutput {
    pub indexed: usize,
}

/// How much of the work this pass found is encoded.
///
/// `total` is counted once, when the pass starts. A Space that is still being written to feeds the
/// same pass more chunks, so `indexed` can pass `total`; both numbers are reported as measured.
/// Bounding the ratio for a progress bar is the presenter's job, not a lie told here.
#[derive(Clone, Copy, Debug, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeIndexingProgress {
    pub indexed: u32,
    pub total: u32,
}

/// A sink for [`KnowledgeIndexingProgress`]. Indexing started by a person is watched in the Task
/// Center; indexing that a derivation needs on the way to its own output reports nothing, because
/// that Run already owns the progress the person is watching.
///
/// A pass is awaited inside a Tauri command and inside a Run step, so the sink crosses `.await`
/// points on a multi-threaded runtime: it is `Send` like everything else the pass holds.
pub(crate) type ProgressSink<'a> = &'a mut (dyn FnMut(KnowledgeIndexingProgress) + Send);

/// A pulled token ends the pass with this, so the command layer can report an outcome instead of
/// a failure.
pub(crate) const CANCELLED: &str = "knowledge-indexing-cancelled";

pub(crate) async fn index_space<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
    progress: ProgressSink<'_>,
) -> Result<IndexOutput, String> {
    if space_id.trim().is_empty() {
        return Err("Knowledge retrieval indexing requires a space id".to_owned());
    }
    let pass = indexing_control::begin(
        &space_id,
        indexing_control::Admission::Requested,
        CancellationToken::new(),
    )
    .await?;
    let resources = preparation::acquire_verified_models(
        &app,
        catalog::TEXT_EMBEDDING_MODELS,
        pass.cancellation(),
    )
    .await?;
    index_with_generation(
        app,
        space_id,
        node_ids,
        resources,
        pass.cancellation(),
        progress,
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
    // A Run that cannot answer without the index waits for whatever pass is already indexing this
    // Space, then fills what its own scope still needs. It stays cancellable through the Run.
    let pass = indexing_control::begin(
        space_id,
        indexing_control::Admission::Required,
        cancellation.clone(),
    )
    .await?;
    index_with_generation(
        app.clone(),
        space_id.to_owned(),
        node_ids,
        resources,
        pass.cancellation(),
        &mut |_| {},
    )
    .await
}

/// Chooses the smallest build that can make the Space searchable again.
///
/// A generation that is still active and still materialised in LanceDB is reused, so only chunks
/// whose embedding is missing or stale are encoded. A new generation — every chunk in the Space,
/// re-encoded — is started only when there is nothing left to reuse: no active generation, a
/// different embedding spec, or an index whose table is gone. The replaced generation keeps
/// answering searches until the new one is complete (`generation::activate`).
async fn index_with_generation<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
    resources: preparation::PreparedModels,
    cancellation: &CancellationToken,
    progress: ProgressSink<'_>,
) -> Result<IndexOutput, String> {
    let pool = db::pool(&app).await?;
    let mut reusable =
        generation::find_active(pool, &space_id, embedding::EMBEDDING_SPEC_ID).await?;
    if reusable.is_some() && !index_is_materialised(&app).await? {
        reusable = None;
    }
    let (generation, node_ids, activate) = match reusable {
        Some(active) => (active, node_ids, false),
        None => (
            generation::begin_rebuild(pool, &space_id, embedding::EMBEDDING_SPEC_ID).await?,
            None,
            true,
        ),
    };
    let indexed = generation_build::run(
        app,
        generation_build::GenerationBuild {
            space_id: &space_id,
            node_ids,
            generation,
            activate,
        },
        resources,
        cancellation,
        progress,
    )
    .await?;
    Ok(IndexOutput { indexed })
}

/// `spaces_chunk_embeddings` only records that a chunk was indexed; the vector itself lives in
/// LanceDB. A lost table therefore leaves markers that claim a complete index no incremental pass
/// would ever fill — the same condition `search` reports as a required rebuild.
pub(super) async fn index_is_materialised<R: Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
    let index = lancedb_store::connect_retrieval_db(app).await?;
    Ok(lancedb_store::open_knowledge_table_if_exists(&index)
        .await?
        .is_some())
}
