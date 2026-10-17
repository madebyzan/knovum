use crate::foundation::app_error::AppError;
pub(crate) mod metadata;
mod persistence;
mod prompt;
mod sources;
mod validation;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use super::source_inputs::SourceInput;
use super::stage;
use crate::ai::{
    capabilities::image,
    contracts::{ExecutionCapability, GeneratedImage},
    settings,
    tasks::{ExecutionSource, TaskExecutionTarget, TaskTargetSelection},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateVisualizationRequest {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<SourceInput>,
    intent: String,
    aspect_ratio: crate::spaces::actions::catalog::parameters::media::ImageAspectRatio,
    target: TaskTargetSelection,
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
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<VisualizationOutput, AppError> {
    let target = settings::resolve_task(
        &app,
        request.target.clone(),
        ExecutionCapability::ImageGeneration,
    )?;
    authority
        .require_active(crate::spaces::db::pool(&app).await?)
        .await?;
    if let Err(error) = persistence::start(&app, &request, &target, &authority).await {
        let settlement = super::failure::fail_pending(
            &app,
            &request.space_id,
            &request.output_node_id,
            super::failure::PendingNodeState::Processing,
        )
        .await;
        return Err(super::failure::with_settlement(error, settlement));
    }
    let generated_visualization = execute(&app, &request, &target, &stages, cancellation).await;
    match generated_visualization {
        Ok(image) => {
            Ok(persistence::complete(&app, &request, image, &authority, cancellation).await?)
        }
        Err(error) => {
            let settlement = persistence::fail(&app, &request.id, &error.message).await;
            Err(super::failure::with_settlement(
                error,
                settlement.map_err(|error| error.to_string()),
            ))
        }
    }
}

/// Images reach the model as pixels, everything else as retrieved text. A
/// selection of images alone is a complete request: there is nothing to index.
async fn execute(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    target: &TaskExecutionTarget,
    stages: &Channel<String>,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<VisualizationContent, AppError> {
    stage(stages, "preparingSources")?;
    let sources = sources::prepare(app, request, target, cancellation).await?;
    if sources.is_empty() {
        return Err("Selected resources contain nothing a model can read".into());
    }
    stage(stages, "generatingImage")?;
    let prompt = prompt::build(
        &request.intent,
        &sources.grounding,
        sources.attached_images(),
    )?;
    let aspect_ratio = request.aspect_ratio.as_str();
    match sources.delivery {
        sources::Delivery::Model { target, references } => image::generate(
            image::ImageGenerationRequest {
                target,
                prompt,
                aspect_ratio: Some(aspect_ratio.to_owned()),
                references,
            },
            cancellation,
        )
        .await
        .map(VisualizationContent::from),
        sources::Delivery::Agent {
            runtime_kind,
            attachments,
            _files,
        } => {
            let bytes = crate::external_agents::task::artifact(app, runtime_kind, format!("Create the requested image as PNG with aspect ratio {aspect_ratio}. Use your own available tools/Skills.\n{prompt}"), attachments, crate::external_agents::task::ArtifactFormat::Png, cancellation.clone()).await?;
            Ok(VisualizationContent {
                bytes,
                execution_source: ExecutionSource::Agent { runtime_kind },
                revised_prompt: None,
            })
        }
    }
}

pub(super) struct VisualizationContent {
    bytes: Vec<u8>,
    execution_source: ExecutionSource,
    revised_prompt: Option<String>,
}

impl From<GeneratedImage> for VisualizationContent {
    fn from(image: GeneratedImage) -> Self {
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
