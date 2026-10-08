use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::{
    completion,
    driver::{self, DriveOutcome},
};
use crate::{
    agent_runtime::protocol::messages::AgentInputMessage,
    spaces::conversations::{
        execution::gateway::Gateway, ConversationTaskFinishedState, ConversationTaskUpdate,
    },
};

pub(super) fn launch(
    host: Gateway,
    messages: Vec<AgentInputMessage>,
    resume: Option<Value>,
    cancellation: CancellationToken,
) {
    let ids = host.ids.clone();
    let pool = host.pool.clone();
    let updates = host.updates.clone();
    let app = host.app.clone();
    let snapshot_id = host.extensions.id.clone();
    tauri::async_runtime::spawn(async move {
        let state = match driver::drive(host, messages, resume, cancellation).await {
            Ok(DriveOutcome::Waiting) => Ok(ConversationTaskFinishedState::Waiting),
            Ok(DriveOutcome::Paused) => Ok(ConversationTaskFinishedState::Paused),
            Ok(DriveOutcome::Completed) => completion::complete(&pool, &ids).await,
            Err(error) => Err(error),
        };
        let (state, terminal_error) = match state {
            Ok(state) => (state, None),
            Err(error) => completion::settle_failure(&pool, &ids.run_id, &error).await,
        };
        let resumable = matches!(
            state,
            ConversationTaskFinishedState::Paused | ConversationTaskFinishedState::Waiting
        );
        if resumable {
            crate::extensions::mcp::sessions::release(&app, &snapshot_id).await;
        } else if let Err(error) = crate::extensions::reclamation::release(&app, &snapshot_id).await
        {
            log::error!("Agent extension release failed: {error}");
        }
        super::cancellation::remove(&ids.run_id);
        if let Some(channel) = updates {
            let _ = channel.send(ConversationTaskUpdate::Finished {
                state,
                error: terminal_error,
            });
        }
    });
}
