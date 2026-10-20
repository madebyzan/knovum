use tauri::{ipc::Channel, AppHandle};

use crate::foundation::app_error::{AppError, AppResult};
pub(crate) mod contracts;
mod execution;
pub(crate) mod import;
pub(crate) mod recovery;
mod session;
pub(crate) mod workspace;

#[tauri::command]
pub async fn create_agent_artifacts(
    app: AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
) -> Result<Vec<crate::ai::contracts::AgentArtifact>, AppError> {
    if request.prompt.trim().is_empty() {
        return Err(AppError::invalid("Agent creation prompt cannot be empty"));
    }
    crate::ai::settings::ensure_agent_enabled(&app, request.kind)?;
    let workspace = crate::spaces::agent_artifacts::workspace::session_directory(
        &app,
        &request.space_id,
        &request.session_id,
    )?;
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
                crate::external_agents::artifacts::discard(&workspace)?;
                return Err(AppError::internal(error));
            }
            if artifacts.is_empty() {
                crate::external_agents::artifacts::discard(&workspace)?;
            }
            Ok(artifacts)
        }
        Err(error) => {
            let persisted = if error.code == "cancelled" {
                session::cancel(&app, &session_id).await
            } else {
                session::fail(&app, &session_id, &error.to_string()).await
            };
            let cleaned = crate::external_agents::artifacts::discard(&workspace);
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
    let workspace =
        crate::spaces::agent_artifacts::workspace::session_directory(&app, &space_id, &session_id)?;
    let artifact = crate::external_agents::artifacts::resolve(&workspace, &relative_path)?;
    crate::desktop::system_preview::open(app, artifact)
        .await
        .map_err(AppError::internal)
}

#[tauri::command]
pub fn discard_artifact_workspace(
    app: AppHandle,
    space_id: String,
    session_id: String,
) -> AppResult<()> {
    let workspace =
        crate::spaces::agent_artifacts::workspace::session_directory(&app, &space_id, &session_id)?;
    crate::external_agents::artifacts::discard(&workspace)
}

#[tauri::command]
pub fn remove_agent_artifact(
    app: AppHandle,
    space_id: String,
    session_id: String,
    relative_path: String,
) -> AppResult<()> {
    let workspace =
        crate::spaces::agent_artifacts::workspace::session_directory(&app, &space_id, &session_id)?;
    crate::external_agents::artifacts::remove(&workspace, &relative_path)
}
