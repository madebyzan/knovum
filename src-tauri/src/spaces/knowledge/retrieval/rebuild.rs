use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::{
    ai::capabilities::embedding,
    local_resources::{models::catalog, preparation},
    spaces::db,
};
use tokio_util::sync::CancellationToken;

use super::{generation, generation_build};

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
    let pool = db::pool(&app).await?;
    let generation =
        generation::begin_rebuild(pool, &space_id, embedding::EMBEDDING_SPEC_ID).await?;
    let indexed = generation_build::run(
        app,
        &space_id,
        None,
        resources,
        generation,
        true,
        &CancellationToken::new(),
    )
    .await?;
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
    index_with_generation(
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
    index_with_generation(
        app.clone(),
        space_id.to_owned(),
        node_ids,
        resources,
        cancellation,
    )
    .await
}

async fn index_with_generation<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
    resources: preparation::PreparedModels,
    cancellation: &CancellationToken,
) -> Result<IndexOutput, String> {
    let pool = db::pool(&app).await?;
    let (generation, node_ids, activate) =
        match generation::active(pool, &space_id, embedding::EMBEDDING_SPEC_ID).await {
            Ok(active) => (active, node_ids, false),
            Err(error) if error == generation::REBUILD_REQUIRED => (
                generation::begin_rebuild(pool, &space_id, embedding::EMBEDDING_SPEC_ID).await?,
                None,
                true,
            ),
            Err(error) => return Err(error),
        };
    let indexed = generation_build::run(
        app,
        &space_id,
        node_ids,
        resources,
        generation,
        activate,
        cancellation,
    )
    .await?;
    Ok(IndexOutput { indexed })
}
