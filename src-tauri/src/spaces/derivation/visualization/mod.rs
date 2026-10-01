use crate::app_error::AppError;
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
        settings,
        tasks::{ExecutionSource, TaskExecutionTarget, TaskTargetSelection},
    },
    spaces::knowledge::retrieval::hybrid::{self, HybridSearchRequest},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateVisualizationRequest {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<VisualizationInput>,
    intent: String,
    style: String,
    aspect_ratio: crate::spaces::actions::catalog::parameters::media::ImageAspectRatio,
    target: TaskTargetSelection,
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

pub(super) async fn generate_for_action(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: GenerateVisualizationRequest,
    stages: Channel<String>,
) -> Result<VisualizationOutput, AppError> {
    generate_inner(authority, app, request, stages).await
}

async fn generate_inner(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: GenerateVisualizationRequest,
    stages: Channel<String>,
) -> Result<VisualizationOutput, AppError> {
    let target = settings::resolve_task(
        &app,
        request.target.clone(),
        ExecutionCapability::ImageGeneration,
    )?;
    let (cancellation, _lease) = crate::ai::executions::acquire(request.id.clone())?;
    authority
        .require_active(crate::spaces::db::pool(&app).await?)
        .await?;
    if let Err(error) = persistence::start(&app, &request, &target, &authority).await {
        super::failure::fail_pending(
            &app,
            &request.space_id,
            &request.output_node_id,
            super::failure::PendingNodeState::Processing,
        )
        .await?;
        return Err(error);
    }
    let generated_visualization = execute(&app, &request, &target, &stages, &cancellation).await;
    match generated_visualization {
        Ok(image) => {
            Ok(persistence::complete(&app, &request, image, &authority, &cancellation).await?)
        }
        Err(error) => {
            persistence::fail(&app, &request.id, &error.message).await?;
            Err(error)
        }
    }
}

async fn execute(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    target: &TaskExecutionTarget,
    stages: &Channel<String>,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<VisualizationResult, AppError> {
    stage(stages, "retrieving")?;
    crate::spaces::knowledge::retrieval::rebuild::prepare_space_index(
        app,
        &request.space_id,
        Some(
            request
                .inputs
                .iter()
                .map(|input| input.node_id.clone())
                .collect(),
        ),
        &request.id,
        cancellation,
    )
    .await?;
    let retrieval = hybrid::search(
        app,
        HybridSearchRequest {
            semantic: true,
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
        cancellation,
    )
    .await?;
    let sources = crate::spaces::knowledge::grounding::prompt::select_sources(retrieval.matches)?;
    if sources.is_empty() {
        return Err("Selected resources contain no current indexed knowledge".into());
    }
    stage(stages, "generatingImage")?;
    let prompt = prompt::build(&request.intent, &request.style, &sources)?;
    match target {
        TaskExecutionTarget::Model(target) => image::generate(
            app,
            image::ImageGenerationRequest {
                target: target.clone(),
                prompt,
                aspect_ratio: request.aspect_ratio.as_str().to_owned(),
            },
            cancellation,
        )
        .await
        .map(VisualizationResult::from),
        TaskExecutionTarget::Agent { runtime_kind } => {
            let bytes = crate::external_agents::task::artifact(app, *runtime_kind, format!("Create the requested image as PNG with aspect ratio {}. Use your own available tools/Skills.\n{prompt}", request.aspect_ratio.as_str()), Vec::new(), crate::external_agents::task::ArtifactFormat::Png, cancellation.clone()).await?;
            Ok(VisualizationResult {
                bytes,
                execution_source: ExecutionSource::Agent {
                    runtime_kind: *runtime_kind,
                },
                revised_prompt: None,
            })
        }
    }
}

pub(super) struct VisualizationResult {
    bytes: Vec<u8>,
    execution_source: ExecutionSource,
    revised_prompt: Option<String>,
}

impl From<image::GeneratedImage> for VisualizationResult {
    fn from(image: image::GeneratedImage) -> Self {
        Self {
            bytes: image.bytes,
            execution_source: ExecutionSource::Model {
                provider: image.provider,
                model: image.model,
            },
            revised_prompt: image.revised_prompt,
        }
    }
}
