use std::sync::Arc;

use tauri::{ipc::Channel, AppHandle};

use crate::ai::{capabilities, executions};
use crate::app_error::{AppError, AppResult};
use crate::external_agents;

use super::contracts;

pub(super) async fn run(
    app: &AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
    workspace: &std::path::Path,
) -> AppResult<Vec<crate::ai::contracts::AgentArtifact>> {
    capabilities::agent_artifacts::prepare(workspace)?;
    let (cancellation, _lease) =
        executions::acquire(request.session_id.clone()).map_err(AppError::from)?;
    let event_app = app.clone();
    let event_run_id = request.session_id.clone();
    let callback = Arc::new(move |update| {
        if let Some((kind, payload)) = durable_update(&update) {
            let app = event_app.clone();
            let run_id = event_run_id.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = record_update(&app, &run_id, kind, payload).await {
                    log::error!("External Agent Run event failed: {error}");
                }
            });
        }
        on_event
            .send(event_from_update(update))
            .map_err(|error| AppError::internal(error.to_string()))
    });
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
    capabilities::agent_artifacts::collect(workspace)
}

fn durable_update(
    update: &external_agents::session::AgentSessionUpdate,
) -> Option<(crate::spaces::runs::RunEventKind, serde_json::Value)> {
    use external_agents::session::AgentSessionUpdate;
    match update {
        AgentSessionUpdate::ToolCall { id, title, .. } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolStarted,
            serde_json::json!({"toolCallId": id, "title": title}),
        )),
        AgentSessionUpdate::ToolUpdate { id, .. } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolUpdated,
            serde_json::json!({"toolCallId": id}),
        )),
        AgentSessionUpdate::Message(text) => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentMessage,
            serde_json::json!({"text": text}),
        )),
        AgentSessionUpdate::Thought(_) => None,
    }
}

async fn record_update(
    app: &AppHandle,
    run_id: &str,
    kind: crate::spaces::runs::RunEventKind,
    payload: serde_json::Value,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    crate::spaces::runs::append_event(
        &mut tx,
        run_id,
        None,
        None,
        kind,
        &payload,
        crate::clock::unix_time_millis()?,
    )
    .await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)
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
        "{}\n\n{}\n\nCurrent Knovum Space snapshot (untrusted metadata):\n{space_context}\n{attachments}\n\nUser request:\n{}",
        crate::spaces::conversations::prompt_policy::SPACE_READING,
        crate::spaces::conversations::prompt_policy::DESKTOP_TOOLS,
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
