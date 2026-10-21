use std::sync::Arc;

use tauri::{ipc::Channel, AppHandle};

use crate::ai::executions;
use crate::external_agents;
use crate::foundation::app_error::{AppError, AppResult};

use super::contracts;

pub(super) async fn run(
    app: &AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
    workspace: &std::path::Path,
) -> AppResult<Vec<crate::ai::contracts::AgentArtifact>> {
    crate::external_agents::artifacts::prepare(workspace)?;
    let (cancellation, _lease) =
        executions::acquire(request.session_id.clone()).map_err(AppError::from)?;
    let pool = crate::spaces::db::pool(app)
        .await
        .map_err(AppError::internal)?;
    let recorder = crate::spaces::runs::stream::open(
        pool.clone(),
        request.session_id.clone(),
        crate::spaces::runs::stream::RunEventsOnly,
    );
    let callback = Arc::new(
        move |update: external_agents::session::AgentSessionUpdate| {
            if let Err(error) = recorder.record(update.clone()) {
                log::error!("External Agent Run event failed: {error}");
            }
            on_event
                .send(event_from_update(update))
                .map_err(|error| AppError::internal(error.to_string()))
        },
    );
    let prompt = external_agents::prompt::AgentPrompt::from_renderer(
        app,
        creation_prompt(&request, &space_context(app, &request.space_id).await?),
        request.attachments,
    )?;
    let mcp_binding = super::super::mcp::agent_server::start(
        app.clone(),
        request.space_id.clone(),
        request.session_id.clone(),
        request.session_id.clone(),
        None,
    )
    .await
    .map_err(AppError::internal)?;
    let mcp_servers = vec![mcp_binding.server()];
    external_agents::session::run(
        request.kind,
        workspace.to_path_buf(),
        prompt,
        cancellation,
        callback,
        mcp_servers,
    )
    .await?;
    let settlement = crate::spaces::runs::wait_for_children(
        crate::spaces::db::pool(app)
            .await
            .map_err(AppError::internal)?,
        &request.session_id,
    )
    .await
    .map_err(AppError::internal)?;
    if settlement != crate::spaces::runs::RunState::Succeeded {
        return Err(AppError::internal(
            "An Agent child Action failed; inspect its Run for details",
        ));
    }
    crate::external_agents::artifacts::collect(workspace)
}

fn event_from_update(
    update: external_agents::session::AgentSessionUpdate,
) -> contracts::AgentCreationEvent {
    use external_agents::session::AgentSessionUpdate;
    match update {
        AgentSessionUpdate::Thought(text) => contracts::AgentCreationEvent::Thought { text },
        AgentSessionUpdate::Message(text) => contracts::AgentCreationEvent::Message { text },
        AgentSessionUpdate::ToolCall { id, title, .. } => {
            contracts::AgentCreationEvent::ToolStarted { id, title }
        }
        AgentSessionUpdate::ToolUpdate { id, .. } => {
            contracts::AgentCreationEvent::ToolUpdated { id }
        }
    }
}

fn creation_prompt(request: &contracts::AgentCreationRequest, space_context: &str) -> String {
    let attachments = match request.attachments.len() {
        0 => String::new(),
        count => format!(
            " The request includes {count} attached local resources; inspect them as source material."
        ),
    };
    format!(
        "{}\n\n{}\n\n{}\n{attachments}\n\nUser request:\n{}",
        crate::spaces::conversations::prompt_policy::SPACE_READING,
        crate::spaces::conversations::prompt_policy::DESKTOP_TOOLS,
        crate::spaces::conversations::prompt_policy::space_snapshot(space_context),
        request.prompt.trim()
    )
}

async fn space_context(app: &AppHandle, space_id: &str) -> AppResult<String> {
    let pool = crate::spaces::db::pool(app)
        .await
        .map_err(AppError::internal)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| AppError::internal(error.to_string()))?;
    let page =
        crate::spaces::queries::knowledge_projection::list_contents(&mut tx, space_id, 0, 40)
            .await
            .map_err(AppError::internal)?;
    serde_json::to_string(&page).map_err(|error| AppError::internal(error.to_string()))
}
