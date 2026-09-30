mod completion;
pub(crate) mod metadata;
mod preparation;
mod prompt;
pub(crate) mod registry;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use crate::ai::{
    capabilities::video::{self, VideoGenerationRequest},
    contracts::ExecutionCapability,
    models::{ExecutionTarget, ExecutionTargetSelection},
    settings,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VideoInput {
    node_id: String,
    role: String,
    source_revision: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateVideoRequest {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<VideoInput>,
    prompt: String,
    aspect_ratio: String,
    duration_seconds: u8,
    target: ExecutionTargetSelection,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOutput {
    node_id: String,
    data: Value,
    relations: Vec<super::DerivationRelation>,
}

pub async fn generate(
    app: AppHandle,
    request: GenerateVideoRequest,
    stages: Channel<String>,
) -> Result<VideoOutput, String> {
    generate_inner(
        super::action_execution::ExecutionAuthority::DirectCommand,
        app,
        request,
        stages,
    )
    .await
}

pub(super) async fn generate_for_action(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: GenerateVideoRequest,
    stages: Channel<String>,
) -> Result<VideoOutput, String> {
    generate_inner(authority, app, request, stages).await
}

async fn generate_inner(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: GenerateVideoRequest,
    stages: Channel<String>,
) -> Result<VideoOutput, String> {
    let (cancellation, _lease) = registry::acquire(&request.id)?;
    authority
        .require_active(crate::spaces::db::pool(&app).await?)
        .await?;
    let output = match settings::resolve(
        &app,
        request.target.clone(),
        ExecutionCapability::VideoGeneration,
    ) {
        Ok(target) => execute(&app, &request, &target, stages, &cancellation, &authority).await,
        Err(error) => Err(error.to_string()),
    };
    output
}

async fn execute(
    app: &AppHandle,
    request: &GenerateVideoRequest,
    target: &ExecutionTarget,
    stages: Channel<String>,
    cancellation: &tokio_util::sync::CancellationToken,
    authority: &super::action_execution::ExecutionAuthority,
) -> Result<VideoOutput, String> {
    let prepared = match preparation::start(app, request, target, authority).await {
        Ok(prepared) => prepared,
        Err(error) => {
            super::failure::fail_pending(
                app,
                &request.space_id,
                &request.output_node_id,
                super::failure::PendingNodeState::Processing,
            )
            .await?;
            return Err(error);
        }
    };
    let generated = video::generate(
        app,
        VideoGenerationRequest {
            target: target.clone(),
            prompt: prepared.provider_prompt,
            ratio: request.aspect_ratio.clone(),
            duration: request.duration_seconds,
            reference_images: prepared.reference_images,
        },
        cancellation,
        move |stage| stages.send(stage.into()).map_err(|error| error.to_string()),
    )
    .await
    .map_err(|error| error.to_string());
    let generated = match generated {
        Ok(value) => value,
        Err(error) => {
            super::failure::fail_message(app, &request.id, &error).await?;
            return Err(error);
        }
    };
    if let Err(error) = tokio::fs::write(&prepared.path, &generated.bytes).await {
        let message = error.to_string();
        super::failure::fail_message(app, &request.id, &message).await?;
        return Err(message);
    }
    let completion = completion::complete(app, request, &prepared.path, generated, authority).await;
    if let Err(error) = &completion {
        let _ = tokio::fs::remove_file(prepared.path).await;
        super::failure::fail_message(app, &request.id, error).await?;
    }
    completion
}
