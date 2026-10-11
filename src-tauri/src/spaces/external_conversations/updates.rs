use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tauri::AppHandle;

use crate::external_agents::session::AgentSessionUpdate;
use crate::foundation::app_error::AppError;

enum UpdateCommand {
    Record(AgentSessionUpdate),
    Flush(tokio::sync::oneshot::Sender<Result<(), String>>),
}

const UPDATE_BUFFER_CAPACITY: usize = 128;
const MAX_UPDATE_TEXT_BYTES: usize = 64 * 1024;

type UpdateSender = tokio::sync::mpsc::Sender<UpdateCommand>;

static ACTIVE_UPDATES: OnceLock<Mutex<HashMap<String, UpdateSender>>> = OnceLock::new();

fn active_updates() -> &'static Mutex<HashMap<String, UpdateSender>> {
    ACTIVE_UPDATES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn flush(session_id: &str) -> impl std::future::Future<Output = Result<(), String>> + Send {
    let sender = (|| {
        active_updates()
            .lock()
            .map_err(|_| "Agent update registry is unavailable")?
            .get(session_id)
            .cloned()
            .ok_or_else(|| "Agent update stream is unavailable".to_owned())
    })();
    async move {
        let sender = sender?;
        let (acknowledge, done) = tokio::sync::oneshot::channel();
        sender
            .send(UpdateCommand::Flush(acknowledge))
            .await
            .map_err(|_| "Agent update stream has closed")?;
        done.await
            .map_err(|_| "Agent update stream stopped before flush".to_owned())?
    }
}

pub fn release(session_id: &str) {
    if let Ok(mut updates) = active_updates().lock() {
        updates.remove(session_id);
    }
}

pub fn callback(
    app: AppHandle,
    conversation_id: String,
    run_id: String,
    channel: Option<tauri::ipc::Channel<crate::spaces::conversations::ConversationTaskUpdate>>,
) -> Result<crate::external_agents::session::AgentUpdateHandler, String> {
    let (sender, mut updates) = tokio::sync::mpsc::channel(UPDATE_BUFFER_CAPACITY);
    active_updates()
        .lock()
        .map_err(|_| "Agent update registry is unavailable")?
        .insert(conversation_id.clone(), sender.clone());
    tauri::async_runtime::spawn(async move {
        let mut failure: Option<String> = None;
        while let Some(command) = updates.recv().await {
            match command {
                UpdateCommand::Record(update) => {
                    if let Err(error) =
                        record(&app, &conversation_id, &run_id, update, channel.as_ref()).await
                    {
                        log::error!("External conversation update failed: {error}");
                        if failure.is_none() {
                            failure = Some(error);
                        }
                    }
                }
                UpdateCommand::Flush(acknowledge) => {
                    let _ = acknowledge.send(failure.take().map_or(Ok(()), Err));
                }
            }
        }
    });
    Ok(std::sync::Arc::new(move |update| {
        enqueue_update(&sender, update)
    }))
}

fn enqueue_update(sender: &UpdateSender, update: AgentSessionUpdate) -> Result<(), AppError> {
    let size = match &update {
        AgentSessionUpdate::Thought(_) => return Ok(()),
        AgentSessionUpdate::Message(text) => text.len(),
        AgentSessionUpdate::ToolCall {
            id,
            title,
            status,
            context,
        } => id.len() + title.len() + status.len() + context.as_ref().map_or(0, String::len),
        AgentSessionUpdate::ToolUpdate {
            id,
            title,
            status,
            context,
        } => {
            id.len()
                + title.as_ref().map_or(0, String::len)
                + status.as_ref().map_or(0, String::len)
                + context.as_ref().map_or(0, String::len)
        }
    };
    if size > MAX_UPDATE_TEXT_BYTES {
        return Err(AppError::invalid(
            "External Agent update exceeds the size limit",
        ));
    }
    sender
        .try_send(UpdateCommand::Record(update))
        .map_err(|error| match error {
            tokio::sync::mpsc::error::TrySendError::Full(_) => {
                AppError::internal("External Agent updates exceeded the processing budget")
            }
            tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                AppError::internal("External conversation update channel closed")
            }
        })
}

async fn record(
    app: &AppHandle,
    conversation_id: &str,
    run_id: &str,
    update: AgentSessionUpdate,
    channel: Option<&tauri::ipc::Channel<crate::spaces::conversations::ConversationTaskUpdate>>,
) -> Result<(), String> {
    let delta = match &update {
        AgentSessionUpdate::Message(text) => Some(text.clone()),
        _ => None,
    };
    let Some((kind, payload, message)) = durable(update) else {
        return Ok(());
    };
    let pool = super::super::db::pool(app).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = super::super::db::begin_write(pool).await?;
    if let Some(message) = message {
        super::repository::append_agent_message_in_transaction(&mut tx, conversation_id, &message)
            .await?;
    }
    if matches!(
        kind,
        crate::spaces::runs::RunEventKind::ExternalAgentToolStarted
    ) {
        super::repository::clear_agent_message_in_transaction(&mut tx, conversation_id).await?;
    }
    let notify_run = matches!(
        kind,
        crate::spaces::runs::RunEventKind::ExternalAgentToolStarted
            | crate::spaces::runs::RunEventKind::ExternalAgentToolUpdated
    );
    crate::spaces::runs::append_event(&mut tx, run_id, None, None, kind, &payload, now).await?;
    tx.commit().await.map_err(super::super::db::sql_error)?;
    if notify_run {
        crate::spaces::runs::announce_change();
    }
    if let (Some(channel), Some(text)) = (channel, delta) {
        let _ = channel.send(
            crate::spaces::conversations::ConversationTaskUpdate::ContentDelta {
                task_id: run_id.to_owned(),
                text,
            },
        );
    }
    if matches!(
        kind,
        crate::spaces::runs::RunEventKind::ExternalAgentToolStarted
    ) {
        if let Some(channel) = channel {
            let _ =
                channel.send(crate::spaces::conversations::ConversationTaskUpdate::ContentBoundary);
        }
    }
    Ok(())
}

fn durable(
    update: AgentSessionUpdate,
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
        AgentSessionUpdate::ToolCall {
            id,
            title,
            status,
            context,
        } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolStarted,
            serde_json::json!({"toolCallId": id, "title": title, "status": status, "context": context}),
            None,
        )),
        AgentSessionUpdate::ToolUpdate {
            id,
            title,
            status,
            context,
        } => Some((
            crate::spaces::runs::RunEventKind::ExternalAgentToolUpdated,
            serde_json::json!({"toolCallId": id, "title": title, "status": status, "context": context}),
            None,
        )),
    }
}

#[cfg(test)]
#[path = "tests/updates.rs"]
mod tests;
