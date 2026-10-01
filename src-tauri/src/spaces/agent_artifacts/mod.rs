use tauri::{ipc::Channel, AppHandle};

use crate::ai::capabilities;
use crate::app_error::{AppError, AppResult};
pub(crate) mod contracts;
mod execution;
pub(crate) mod import;
pub(crate) mod recovery;
mod session;

#[tauri::command]
pub async fn create_agent_artifacts(
    app: AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
) -> Result<Vec<crate::ai::contracts::AgentArtifact>, AppError> {
    create_agent_artifacts_scoped(app, request, on_event).await
}

async fn create_agent_artifacts_scoped(
    app: AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
) -> Result<Vec<crate::ai::contracts::AgentArtifact>, AppError> {
    if request.prompt.trim().is_empty() {
        return Err(AppError::invalid("Agent creation prompt cannot be empty"));
    }
    crate::ai::settings::ensure_agent_enabled(&app, request.kind)?;
    let workspace =
        capabilities::agent_artifacts::workspace(&app, &request.space_id, &request.session_id)?;
    session::start(
        &app,
        &request.session_id,
        &request.space_id,
        request.kind,
        &request.prompt,
    )
    .await
    .map_err(AppError::internal)?;
    let session_id = request.session_id.clone();
    match execution::run(&app, request, on_event, &workspace).await {
        Ok(artifacts) => {
            if let Err(error) = session::succeed(&app, &session_id).await {
                capabilities::agent_artifacts::discard(&workspace)?;
                return Err(AppError::internal(error));
            }
            if artifacts.is_empty() {
                capabilities::agent_artifacts::discard(&workspace)?;
            }
            Ok(artifacts)
        }
        Err(error) => {
            let persisted = if error.code == "cancelled" {
                session::cancel(&app, &session_id).await
            } else {
                session::fail(&app, &session_id, &error.to_string()).await
            };
            let cleaned = capabilities::agent_artifacts::discard(&workspace);
            persisted.map_err(AppError::internal)?;
            cleaned?;
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn preview_generated_artifact(
    app: AppHandle,
    space_id: String,
    session_id: String,
    relative_path: String,
) -> AppResult<()> {
    let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)?;
    let artifact = capabilities::agent_artifacts::resolve(&workspace, &relative_path)?;
    crate::system_preview::open(app, artifact)
        .await
        .map_err(AppError::internal)
}

#[tauri::command]
pub fn discard_artifact_workspace(
    app: AppHandle,
    space_id: String,
    session_id: String,
) -> AppResult<()> {
    let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)?;
    capabilities::agent_artifacts::discard(&workspace)
}

#[tauri::command]
pub fn consume_agent_artifact(
    app: AppHandle,
    space_id: String,
    session_id: String,
    relative_path: String,
) -> AppResult<()> {
    let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)?;
    capabilities::agent_artifacts::consume(&workspace, &relative_path)
}
