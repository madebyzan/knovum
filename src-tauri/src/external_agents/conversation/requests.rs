use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::external_agents::session::AgentUpdateHandler;
use crate::foundation::app_error::{AppError, AppResult};

/// One prompt and what belongs to that turn alone: where its updates go and what stops it.
pub(super) struct TurnRequest {
    pub(super) prompt: super::super::prompt::AgentPrompt,
    pub(super) on_update: AgentUpdateHandler,
    pub(super) cancellation: CancellationToken,
    pub(super) reply: oneshot::Sender<AppResult<()>>,
}

struct ConversationHandle {
    requests: mpsc::Sender<TurnRequest>,
    cancellation: CancellationToken,
    inactive: CancellationToken,
}

static ACTIVE: OnceLock<Mutex<HashMap<String, ConversationHandle>>> = OnceLock::new();

pub(super) fn register(
    runtime_id: &str,
    cancellation: CancellationToken,
) -> AppResult<mpsc::Receiver<TurnRequest>> {
    let (requests, receiver) = mpsc::channel(1);
    active()
        .lock()
        .map_err(|_| AppError::internal("ACP conversation registry unavailable"))?
        .insert(
            runtime_id.to_owned(),
            ConversationHandle {
                requests,
                cancellation,
                inactive: CancellationToken::new(),
            },
        );
    Ok(receiver)
}

/// Sends one turn to a running runtime and waits for it to end. Cancelling `cancellation` stops
/// the turn; the runtime stays up for the next one.
pub async fn send(
    runtime_id: &str,
    prompt: super::super::prompt::AgentPrompt,
    on_update: AgentUpdateHandler,
    cancellation: CancellationToken,
) -> AppResult<()> {
    let (requests, _) = handle(runtime_id)?;
    let (reply, outcome) = oneshot::channel();
    requests
        .send(TurnRequest {
            prompt,
            on_update,
            cancellation,
            reply,
        })
        .await
        .map_err(|_| AppError::internal("ACP conversation channel closed"))?;
    outcome
        .await
        .map_err(|_| AppError::internal("ACP conversation turn ended without a result"))?
}

pub async fn close(runtime_id: &str) -> Result<(), String> {
    let (cancellation, inactive) = active()
        .lock()
        .map_err(|_| "ACP conversation registry unavailable")?
        .get(runtime_id)
        .map(|handle| (handle.cancellation.clone(), handle.inactive.clone()))
        .ok_or("ACP conversation is disconnected")?;
    cancellation.cancel();
    wait_until_inactive(inactive).await
}

async fn wait_until_inactive(inactive: CancellationToken) -> Result<(), String> {
    tokio::time::timeout(
        super::super::connection::CANCEL_TIMEOUT,
        inactive.cancelled(),
    )
    .await
    .map_err(|_| "ACP conversation did not close before the deadline")?;
    Ok(())
}

pub fn is_active(runtime_id: &str) -> bool {
    active()
        .lock()
        .is_ok_and(|items| items.contains_key(runtime_id))
}

pub(super) fn remove(runtime_id: &str) {
    if let Ok(mut conversations) = active().lock() {
        if let Some(handle) = conversations.remove(runtime_id) {
            handle.inactive.cancel();
        }
    }
}

fn handle(runtime_id: &str) -> AppResult<(mpsc::Sender<TurnRequest>, CancellationToken)> {
    let conversations = active()
        .lock()
        .map_err(|_| AppError::internal("ACP conversation registry unavailable"))?;
    conversations
        .get(runtime_id)
        .map(|handle| (handle.requests.clone(), handle.inactive.clone()))
        .ok_or_else(|| AppError::invalid("ACP conversation is disconnected"))
}

fn active() -> &'static Mutex<HashMap<String, ConversationHandle>> {
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}
