use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::external_agents::session::AgentSessionUpdate;
use crate::spaces::conversations::ConversationTaskUpdate;
use crate::spaces::runs::{
    stream::{self, Sink, Streamed},
    RunEventKind,
};

static ACTIVE_UPDATES: OnceLock<Mutex<HashMap<String, stream::Stream>>> = OnceLock::new();

fn active_updates() -> &'static Mutex<HashMap<String, stream::Stream>> {
    ACTIVE_UPDATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn recorder(session_id: &str) -> Result<stream::Stream, String> {
    active_updates()
        .lock()
        .map_err(|_| "Agent update registry is unavailable")?
        .get(session_id)
        .cloned()
        .ok_or_else(|| "Agent update stream is unavailable".to_owned())
}

pub fn flush(session_id: &str) -> impl std::future::Future<Output = Result<(), String>> + Send {
    let recorder = recorder(session_id);
    async move { recorder?.flush().await }
}

pub fn release(session_id: &str) {
    if let Ok(mut updates) = active_updates().lock() {
        updates.remove(session_id);
    }
}

/// The conversation an Agent's updates belong to, and the assistant message they accumulate in.
struct Conversation {
    conversation_id: String,
}

impl Sink for Conversation {
    async fn settle(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        batch: &[Streamed],
    ) -> Result<(), String> {
        let Some(write) = message_write(batch) else {
            return Ok(());
        };
        super::repository::write_agent_message_in_transaction(tx, &self.conversation_id, write)
            .await
    }
}

/// The net effect a batch has on the assistant message: a tool call ends the commentary that
/// preceded it, so only the text after the last one survives.
fn message_write(batch: &[Streamed]) -> Option<super::repository::AgentMessageWrite> {
    let started = batch
        .iter()
        .rposition(|event| matches!(event.kind, RunEventKind::ExternalAgentToolStarted));
    let commentary: String = batch[started.map_or(0, |at| at + 1)..]
        .iter()
        .filter(|event| matches!(event.kind, RunEventKind::ExternalAgentMessage))
        .filter_map(Streamed::text)
        .collect();
    match started {
        Some(_) => Some(super::repository::AgentMessageWrite::Replace(commentary)),
        None => (!commentary.is_empty())
            .then_some(super::repository::AgentMessageWrite::Append(commentary)),
    }
}

pub fn callback(
    pool: &SqlitePool,
    conversation_id: String,
    run_id: String,
    channel: Option<tauri::ipc::Channel<ConversationTaskUpdate>>,
) -> Result<crate::external_agents::session::AgentUpdateHandler, String> {
    let recorder = stream::open(
        pool.clone(),
        run_id.clone(),
        Conversation {
            conversation_id: conversation_id.clone(),
        },
    );
    active_updates()
        .lock()
        .map_err(|_| "Agent update registry is unavailable")?
        .insert(conversation_id, recorder.clone());
    Ok(std::sync::Arc::new(move |update| {
        show(&run_id, channel.as_ref(), &update);
        recorder.record(update)
    }))
}

/// The screen follows the stream itself, not its durability: a chunk is shown as it arrives
/// and recorded with the batch it lands in.
fn show(
    run_id: &str,
    channel: Option<&tauri::ipc::Channel<ConversationTaskUpdate>>,
    update: &AgentSessionUpdate,
) {
    let Some(channel) = channel else {
        return;
    };
    match update {
        AgentSessionUpdate::Message(text) => {
            let _ = channel.send(ConversationTaskUpdate::ContentDelta {
                task_id: run_id.to_owned(),
                text: text.clone(),
            });
        }
        AgentSessionUpdate::ToolCall { .. } => {
            let _ = channel.send(ConversationTaskUpdate::ContentBoundary);
        }
        AgentSessionUpdate::ToolUpdate { .. } => {}
    }
}

#[cfg(test)]
#[path = "tests/updates.rs"]
mod tests;
