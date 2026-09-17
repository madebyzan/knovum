use std::sync::Arc;

use tauri::{ipc::Channel, AppHandle};

use crate::ai::{
    capabilities, contracts,
    error::{AiError, AiErrorCode, AiResult},
    executions, providers,
};

pub(crate) mod import;
pub(crate) mod recovery;

#[tauri::command]
pub async fn create_agent_artifacts(
    app: AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
) -> Result<Vec<contracts::AgentArtifact>, AiError> {
    if request.prompt.trim().is_empty() {
        return Err(AiError::invalid("Agent creation prompt cannot be empty"));
    }
    crate::ai::settings::ensure_agent_enabled(&app, request.kind)?;
    let workspace =
        capabilities::agent_artifacts::workspace(&app, &request.space_id, &request.session_id)?;
    super::derivation::workflow_runs::agent_runs::start(
        &app,
        &request.session_id,
        &request.space_id,
        request.kind,
    )
    .await
    .map_err(AiError::internal)?;
    let session_id = request.session_id.clone();
    match execute_creation(&app, request, on_event, &workspace).await {
        Ok(artifacts) => {
            if let Err(error) =
                super::derivation::workflow_runs::agent_runs::succeed(&app, &session_id).await
            {
                capabilities::agent_artifacts::discard(&workspace)?;
                return Err(AiError::internal(error));
            }
            if artifacts.is_empty() {
                capabilities::agent_artifacts::discard(&workspace)?;
            }
            Ok(artifacts)
        }
        Err(error) => {
            let persisted = if error.code == AiErrorCode::Cancelled {
                super::derivation::workflow_runs::agent_runs::cancel(&app, &session_id).await
            } else {
                super::derivation::workflow_runs::agent_runs::fail(
                    &app,
                    &session_id,
                    &error.to_string(),
                )
                .await
            };
            let cleaned = capabilities::agent_artifacts::discard(&workspace);
            persisted.map_err(AiError::internal)?;
            cleaned?;
            Err(error)
        }
    }
}

async fn execute_creation(
    app: &AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
    workspace: &std::path::Path,
) -> AiResult<Vec<contracts::AgentArtifact>> {
    capabilities::agent_artifacts::prepare(workspace)?;
    let (cancellation, _lease) =
        executions::acquire(request.session_id.clone()).map_err(AiError::from)?;
    let callback = Arc::new(move |update| {
        use providers::agents::session::AgentSessionUpdate;
        let event = match update {
            AgentSessionUpdate::Thought(text) => contracts::AgentCreationEvent::Thought { text },
            AgentSessionUpdate::Message(text) => contracts::AgentCreationEvent::Message { text },
            AgentSessionUpdate::ToolCall { id, title } => {
                contracts::AgentCreationEvent::ToolStarted { id, title }
            }
            AgentSessionUpdate::ToolUpdate { id } => {
                contracts::AgentCreationEvent::ToolUpdated { id }
            }
        };
        on_event
            .send(event)
            .map_err(|error| AiError::internal(error.to_string()))
    });
    let prompt = providers::agents::prompt::AgentPrompt::from_renderer(
        app,
        creation_prompt(
            &request.space_id,
            &request.prompt,
            request.attachments.len(),
        ),
        request.attachments,
    )?;
    let knowledge = super::knowledge::mcp::agent_server::start(
        app.clone(),
        request.space_id.clone(),
        request.kind.as_str().to_owned(),
    )
    .await
    .map_err(AiError::internal)?;
    providers::agents::session::run(
        request.kind,
        workspace.to_path_buf(),
        prompt,
        cancellation,
        callback,
        vec![knowledge.server()],
    )
    .await?;
    providers::agents::prompt::discard_staged_context(workspace).await?;
    let artifacts = capabilities::agent_artifacts::collect(workspace)?;
    Ok(artifacts)
}

#[tauri::command]
pub async fn preview_generated_artifact(
    app: AppHandle,
    space_id: String,
    session_id: String,
    relative_path: String,
) -> AiResult<()> {
    let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)?;
    let artifact = capabilities::agent_artifacts::resolve(&workspace, &relative_path)?;
    crate::system_preview::open(app, artifact)
        .await
        .map_err(AiError::internal)
}

#[tauri::command]
pub fn discard_artifact_workspace(
    app: AppHandle,
    space_id: String,
    session_id: String,
) -> AiResult<()> {
    let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)?;
    capabilities::agent_artifacts::discard(&workspace)
}

#[tauri::command]
pub fn consume_agent_artifact(
    app: AppHandle,
    space_id: String,
    session_id: String,
    relative_path: String,
) -> AiResult<()> {
    let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)?;
    capabilities::agent_artifacts::consume(&workspace, &relative_path)
}

fn creation_prompt(space_id: &str, user_prompt: &str, attachment_count: usize) -> String {
    let attachments = match attachment_count {
        0 => String::new(),
        count => format!(
            " The request includes {count} attached local resources; inspect them as source material."
        ),
    };
    format!(
        "Work only inside the current working directory. Save every final artifact the user requested as a file in this directory. Do not write temporary dependencies, caches, or project scaffolding here unless they are themselves requested deliverables. The attached Knovum Knowledge MCP server is scoped by tool arguments; use Space ID `{}` for every knowledge read or write in this session. Commit knowledge changes only when they are part of the requested result.{}\n\nUser request:\n{}",
        space_id,
        attachments,
        user_prompt.trim()
    )
}
