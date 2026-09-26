use super::contracts;
use std::sync::Arc;

use tauri::{ipc::Channel, AppHandle};

use crate::ai::{
    capabilities,
    error::{AiError, AiResult},
    executions,
};
use crate::external_agents;

pub(super) async fn run(
    app: &AppHandle,
    request: contracts::AgentCreationRequest,
    on_event: Channel<contracts::AgentCreationEvent>,
    workspace: &std::path::Path,
) -> AiResult<Vec<crate::ai::contracts::AgentArtifact>> {
    capabilities::agent_artifacts::prepare(workspace)?;
    let (cancellation, _lease) =
        executions::acquire(request.session_id.clone()).map_err(AiError::from)?;
    let event_app = app.clone();
    let event_run_id = request.session_id.clone();
    let callback = Arc::new(move |update| {
        if let Some((kind, payload)) = durable_update(&update, &event_run_id) {
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
            .map_err(|error| AiError::internal(error.to_string()))
    });
    let prompt = external_agents::prompt::AgentPrompt::from_renderer(
        app,
        creation_prompt(&request),
        request.attachments,
    )?;
    let mcp_binding = super::super::mcp::agent_server::start(
        app.clone(),
        request.space_id.clone(),
        request.kind.as_str().to_owned(),
    )
    .await
    .map_err(AiError::internal)?;
    let mcp_servers = vec![mcp_binding.server()];
    external_agents::session::run(
        request.session_id.clone(),
        request.kind,
        workspace.to_path_buf(),
        prompt,
        cancellation,
        callback,
        mcp_servers,
    )
    .await?;
    external_agents::prompt::discard_staged_context(workspace).await?;
    capabilities::agent_artifacts::collect(workspace)
}

fn durable_update(
    update: &external_agents::session::AgentSessionUpdate,
    session_id: &str,
) -> Option<(crate::spaces::runs::RunEventKind, serde_json::Value)> {
    use external_agents::session::AgentSessionUpdate;
    match update {
        AgentSessionUpdate::ToolCall { id, title } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolStarted,
            serde_json::json!({"toolCallId": id, "title": title}),
        )),
        AgentSessionUpdate::ToolUpdate { id } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolUpdated,
            serde_json::json!({"toolCallId": id}),
        )),
        AgentSessionUpdate::PermissionRequested {
            request_id,
            title,
            options,
        } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentPermissionRequested,
            serde_json::json!({
                "requestId": request_id,
                "sessionId": session_id,
                "title": title,
                "options": options.iter().map(|option| serde_json::json!({
                    "id": option.id, "label": option.label, "kind": option.kind,
                })).collect::<Vec<_>>(),
            }),
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
        AgentSessionUpdate::ToolCall { id, title } => {
            contracts::AgentCreationEvent::ToolStarted { id, title }
        }
        AgentSessionUpdate::ToolUpdate { id } => contracts::AgentCreationEvent::ToolUpdated { id },
        AgentSessionUpdate::PermissionRequested {
            request_id,
            title,
            options,
        } => contracts::AgentCreationEvent::PermissionRequested {
            request_id,
            title,
            options: options
                .into_iter()
                .map(|option| contracts::AgentPermissionOption {
                    id: option.id,
                    label: option.label,
                    kind: option.kind.to_owned(),
                })
                .collect(),
        },
    }
}

fn creation_prompt(request: &contracts::AgentCreationRequest) -> String {
    let attachments = match request.attachments.len() {
        0 => String::new(),
        count => format!(
            " The request includes {count} attached local resources; inspect them as source material."
        ),
    };
    let knowledge = format!(" The attached Knovum Knowledge MCP server is scoped by tool arguments; use Space ID `{}` for every knowledge read or write in this session. Commit knowledge changes only when they are part of the requested result. A commit may return requires_approval; wait for the user to approve it, then retry the unchanged request with the same idempotency key and returned approvalId.", request.space_id);
    format!(
        "Work only inside the current working directory. Save every final artifact the user requested as a file in this directory. Do not write temporary dependencies, caches, or project scaffolding here unless they are themselves requested deliverables.{knowledge}{attachments}\n\nUser request:\n{}",
        request.prompt.trim()
    )
}
