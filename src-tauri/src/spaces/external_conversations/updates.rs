use tauri::AppHandle;

use crate::{ai::error::AiError, external_agents::session::AgentSessionUpdate};

pub fn callback(
    app: AppHandle,
    conversation_id: String,
    run_id: String,
) -> crate::external_agents::session::AgentUpdateHandler {
    let (sender, mut updates) = tokio::sync::mpsc::unbounded_channel();
    tauri::async_runtime::spawn(async move {
        while let Some(update) = updates.recv().await {
            if let Err(error) = record(&app, &conversation_id, &run_id, update).await {
                log::error!("External conversation update failed: {error}");
            }
        }
    });
    std::sync::Arc::new(move |update| {
        sender
            .send(update)
            .map_err(|_| AiError::internal("External conversation update channel closed"))
    })
}

async fn record(
    app: &AppHandle,
    conversation_id: &str,
    run_id: &str,
    update: AgentSessionUpdate,
) -> Result<(), String> {
    let Some((kind, payload, message)) = durable(update, conversation_id) else {
        return Ok(());
    };
    let pool = super::super::db::pool(app).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = super::super::db::begin_write(pool).await?;
    if let Some(message) = message {
        super::repository::append_agent_message_in_transaction(
            &mut tx,
            conversation_id,
            &message,
            now,
        )
        .await?;
    }
    crate::spaces::runs::append_event(&mut tx, run_id, None, None, kind, &payload, now).await?;
    tx.commit().await.map_err(super::super::db::sql_error)
}

fn durable(
    update: AgentSessionUpdate,
    conversation_id: &str,
) -> Option<(
    crate::spaces::runs::RunEventKind,
    serde_json::Value,
    Option<String>,
)> {
    match update {
        AgentSessionUpdate::Thought(_) => None,
        AgentSessionUpdate::Message(text) => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentMessage,
            serde_json::json!({"text": text}),
            Some(text),
        )),
        AgentSessionUpdate::ToolCall { id, title } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolStarted,
            serde_json::json!({"toolCallId": id, "title": title}),
            None,
        )),
        AgentSessionUpdate::ToolUpdate { id } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolUpdated,
            serde_json::json!({"toolCallId": id}),
            None,
        )),
        AgentSessionUpdate::PermissionRequested {
            request_id,
            title,
            options,
        } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentPermissionRequested,
            serde_json::json!({
                "requestId": request_id, "sessionId": conversation_id, "title": title,
                "options": options.into_iter().map(|option| serde_json::json!({
                    "id": option.id, "label": option.label, "kind": option.kind,
                })).collect::<Vec<_>>(),
            }),
            None,
        )),
    }
}
