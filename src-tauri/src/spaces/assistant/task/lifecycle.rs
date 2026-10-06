use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::{
    completion,
    driver::{self, DriveOutcome},
};
use crate::{
    agent_runtime::protocol::messages::AssistantInputMessage,
    spaces::assistant::{
        execution::gateway::Gateway, AssistantTaskFinishedState, AssistantTaskUpdate,
    },
};

pub(super) fn launch(
    host: Gateway,
    messages: Vec<AssistantInputMessage>,
    resume: Option<Value>,
    cancellation: CancellationToken,
) {
    let ids = host.ids.clone();
    let pool = host.pool.clone();
    let updates = host.updates.clone();
    tauri::async_runtime::spawn(async move {
        let state = match driver::drive(host, messages, resume, cancellation).await {
            Ok(DriveOutcome::Paused) => Ok(AssistantTaskFinishedState::Paused),
            Ok(DriveOutcome::Completed) => completion::complete(&pool, &ids).await,
            Err(error) => Err(error),
        };
        let state = match state {
            Ok(state) => state,
            Err(error) => {
                if let Err(write_error) = completion::fail(&pool, &ids.run_id, &error).await {
                    log::error!("Assistant failure recording failed: {write_error}");
                }
                match completion::stopped_state(&pool, &ids.run_id).await {
                    Ok(state) => state,
                    Err(error) => {
                        log::error!("Assistant final state unavailable: {error}");
                        super::cancellation::remove(&ids.run_id);
                        return;
                    }
                }
            }
        };
        super::cancellation::remove(&ids.run_id);
        if let Some(channel) = updates {
            let _ = channel.send(AssistantTaskUpdate::Finished { state });
        }
    });
}
