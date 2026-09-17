pub(crate) mod metadata;
mod persistence;
mod prompt;
mod validation;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use super::stage;
use crate::{
    ai::{
        capabilities::image,
        contracts::ExecutionCapability,
        models::{ExecutionTarget, ExecutionTargetSelection},
        settings,
    },
    spaces::knowledge::retrieval::hybrid::{self, HybridSearchRequest},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateVisualizationRequest {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<VisualizationInput>,
    intent: String,
    style: String,
    aspect_ratio: String,
    target: ExecutionTargetSelection,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VisualizationInput {
    node_id: String,
    role: String,
    source_revision: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualizationOutput {
    node_id: String,
    data: Value,
    relations: Vec<super::DerivationRelation>,
}

pub async fn create(
    app: AppHandle,
    request: CreateVisualizationRequest,
    stages: Channel<String>,
) -> Result<VisualizationOutput, String> {
    let target = settings::resolve(
        &app,
        request.target.clone(),
        ExecutionCapability::ImageGeneration,
    )
    .map_err(|error| error.to_string())?;
    let (cancellation, _lease) =
        crate::ai::executions::acquire(request.id.clone()).map_err(|error| error.to_string())?;
    if let Err(error) = persistence::start(&app, &request, &target).await {
        super::failure::fail_pending(
            &app,
            &request.space_id,
            &request.output_node_id,
            super::failure::PendingNodeState::Processing,
        )
        .await?;
        return Err(error);
    }
    let execution = execute(&app, &request, &target, &stages, &cancellation);
    let result = tokio::select! {
        _ = cancellation.cancelled() => Err("Knowledge visualization was cancelled".to_owned()),
        result = execution => result,
    };
    match result {
        Ok(image) => persistence::complete(&app, &request, image).await,
        Err(error) => {
            persistence::fail(&app, &request.id, &error).await?;
            Err(error)
        }
    }
}

async fn execute(
    app: &AppHandle,
    request: &CreateVisualizationRequest,
    target: &ExecutionTarget,
    stages: &Channel<String>,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<image::GeneratedImage, String> {
    stage(stages, "retrieving")?;
    let retrieval = hybrid::search(
        app,
        HybridSearchRequest {
            space_id: Some(request.space_id.clone()),
            query: request.intent.clone(),
            node_ids: Some(
                request
                    .inputs
                    .iter()
                    .map(|input| input.node_id.clone())
                    .collect(),
            ),
        },
    )
    .await?;
    let sources = crate::spaces::knowledge::grounding::prompt::select_sources(retrieval.matches)?;
    if sources.is_empty() {
        return Err("Selected resources contain no current indexed knowledge".into());
    }
    stage(stages, "generatingImage")?;
    image::generate(
        app,
        image::ImageGenerationRequest {
            target: target.clone(),
            prompt: prompt::build(&request.intent, &request.style, &sources)?,
            aspect_ratio: request.aspect_ratio.clone(),
        },
        cancellation,
    )
    .await
    .map_err(|error| error.to_string())
}
