use crate::app_error::AppError;
mod completion;
pub(crate) mod metadata;
mod preparation;
mod prompt;
pub(crate) mod registry;
#[cfg(test)]
mod tests;
mod validation;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use crate::ai::{
    capabilities::video::{self, VideoGenerationRequest},
    contracts::{ExecutionCapability, GeneratedVideo},
    settings,
    tasks::{ExecutionSource, TaskExecutionTarget, TaskTargetSelection},
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
    aspect_ratio: crate::spaces::actions::catalog::parameters::media::VideoAspectRatio,
    duration_seconds: u8,
    target: TaskTargetSelection,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOutput {
    node_id: String,
    data: Value,
    relations: Vec<super::DerivationRelation>,
}

pub(super) async fn generate_for_action(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: GenerateVideoRequest,
    stages: Channel<String>,
) -> Result<VideoOutput, AppError> {
    let (cancellation, _lease) = registry::acquire(&request.id)?;
    authority
        .require_active(crate::spaces::db::pool(&app).await?)
        .await?;
    match settings::resolve_task(
        &app,
        request.target.clone(),
        ExecutionCapability::VideoGeneration,
    ) {
        Ok(target) => execute(&app, &request, &target, stages, &cancellation, &authority).await,
        Err(error) => Err(error),
    }
}

async fn execute(
    app: &AppHandle,
    request: &GenerateVideoRequest,
    target: &TaskExecutionTarget,
    stages: Channel<String>,
    cancellation: &tokio_util::sync::CancellationToken,
    authority: &super::action_execution::ExecutionAuthority,
) -> Result<VideoOutput, AppError> {
    let prepared = match preparation::start(app, request, target, authority).await {
        Ok(prepared) => prepared,
        Err(error) => {
            let settlement = super::failure::fail_pending(
                app,
                &request.space_id,
                &request.output_node_id,
                super::failure::PendingNodeState::Processing,
            )
            .await;
            return Err(super::failure::with_settlement(error.into(), settlement));
        }
    };
    let generated = match target {
        TaskExecutionTarget::Model(model) => video::generate(
            app,
            VideoGenerationRequest {
                target: model.clone(),
                prompt: prepared.provider_prompt,
                ratio: request.aspect_ratio.as_str().to_owned(),
                duration: request.duration_seconds,
                reference_images: prepared.reference_images,
            },
            cancellation,
            move |stage| stages.send(stage.into()).map_err(|error| error.to_string()),
        )
        .await
        .map(VideoContent::from),
        TaskExecutionTarget::Agent { runtime_kind } => {
            super::stage(&stages, "generatingVideo")?;
            crate::external_agents::task::artifact(app, *runtime_kind,
                format!("Produce an MP4 video, requested duration {} seconds and ratio {}, using your own tools/Skills.\n{}", request.duration_seconds, request.aspect_ratio.as_str(), prepared.provider_prompt),
                prepared.reference_images.iter().map(std::path::PathBuf::from).collect(),
                crate::external_agents::task::ArtifactFormat::Mp4, cancellation.clone(),
            ).await.map(|bytes| VideoContent { bytes, execution_source: target.source(), operation_id: None })
        }
    };
    let generated = match generated {
        Ok(value) => value,
        Err(error) => {
            let settlement = super::failure::fail_message(app, &request.id, &error.message).await;
            return Err(super::failure::with_settlement(error, settlement));
        }
    };
    if let Err(error) = tokio::fs::write(&prepared.path, &generated.bytes).await {
        let message = error.to_string();
        return Err(fail_output(app, &request.id, &prepared.path, message.into()).await);
    }
    let completion = async {
        validation::validate(app, &prepared.path, cancellation).await?;
        completion::complete(
            app,
            request,
            &prepared.path,
            generated,
            authority,
            cancellation,
        )
        .await
    }
    .await;
    match completion {
        Ok(output) => Ok(output),
        Err(error) => Err(fail_output(app, &request.id, &prepared.path, error).await),
    }
}

pub(super) struct VideoContent {
    bytes: Vec<u8>,
    execution_source: ExecutionSource,
    operation_id: Option<String>,
}
impl From<GeneratedVideo> for VideoContent {
    fn from(value: GeneratedVideo) -> Self {
        Self {
            bytes: value.bytes,
            execution_source: ExecutionSource::Model {
                provider: value.provider,
                model: value.model,
            },
            operation_id: Some(value.operation_id),
        }
    }
}

async fn fail_output(app: &AppHandle, id: &str, path: &str, mut primary: AppError) -> AppError {
    if let Err(cleanup) = tokio::fs::remove_file(path).await {
        if cleanup.kind() != std::io::ErrorKind::NotFound {
            primary.message = format!(
                "{}; derived video cleanup also failed: {cleanup}",
                primary.message
            );
        }
    }
    let settlement = super::failure::fail_message(app, id, &primary.message).await;
    super::failure::with_settlement(primary, settlement)
}
