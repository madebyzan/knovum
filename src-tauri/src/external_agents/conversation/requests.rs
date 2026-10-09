use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::app_error::{AppError, AppResult};

pub(super) struct TurnRequest {
    pub(super) prompt: super::super::prompt::AgentPrompt,
    pub(super) reply: oneshot::Sender<AppResult<()>>,
}

pub(super) enum ConversationRequest {
    Turn(TurnRequest),
    Finish,
}

struct ConversationHandle {
    requests: mpsc::Sender<ConversationRequest>,
    cancellation: CancellationToken,
    inactive: CancellationToken,
}

static ACTIVE: OnceLock<Mutex<HashMap<String, ConversationHandle>>> = OnceLock::new();

pub(super) fn register(
    id: &str,
    cancellation: CancellationToken,
) -> AppResult<mpsc::Receiver<ConversationRequest>> {
    let (requests, receiver) = mpsc::channel(1);
    active()
        .lock()
        .map_err(|_| AppError::internal("ACP conversation registry unavailable"))?
        .insert(
            id.to_owned(),
            ConversationHandle {
                requests,
                cancellation,
                inactive: CancellationToken::new(),
            },
        );
    Ok(receiver)
}

pub async fn send(id: &str, prompt: super::super::prompt::AgentPrompt) -> AppResult<()> {
    let requests = sender(id)?;
    let (reply, outcome) = oneshot::channel();
    requests
        .send(ConversationRequest::Turn(TurnRequest { prompt, reply }))
        .await
        .map_err(|_| AppError::internal("ACP conversation channel closed"))?;
    outcome
        .await
        .map_err(|_| AppError::internal("ACP conversation turn ended without a result"))?
}

pub async fn close(id: &str) -> Result<(), String> {
    let (cancellation, inactive) = active()
        .lock()
        .map_err(|_| "ACP conversation registry unavailable")?
        .get(id)
        .map(|handle| (handle.cancellation.clone(), handle.inactive.clone()))
        .ok_or("ACP conversation is disconnected")?;
    cancellation.cancel();
    wait_until_inactive(inactive).await
}

pub async fn finish(id: &str) -> Result<(), String> {
    let (requests, inactive) = handle(id).map_err(|error| error.to_string())?;
    requests
        .send(ConversationRequest::Finish)
        .await
        .map_err(|_| "ACP conversation channel closed".to_owned())?;
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

pub fn is_active(id: &str) -> bool {
    active().lock().is_ok_and(|items| items.contains_key(id))
}

pub(super) fn remove(id: &str) {
    if let Ok(mut conversations) = active().lock() {
        if let Some(handle) = conversations.remove(id) {
            handle.inactive.cancel();
        }
    }
}

fn sender(id: &str) -> AppResult<mpsc::Sender<ConversationRequest>> {
    handle(id).map(|(requests, _)| requests)
}

fn handle(id: &str) -> AppResult<(mpsc::Sender<ConversationRequest>, CancellationToken)> {
    let conversations = active()
        .lock()
        .map_err(|_| AppError::internal("ACP conversation registry unavailable"))?;
    conversations
        .get(id)
        .map(|handle| (handle.requests.clone(), handle.inactive.clone()))
        .ok_or_else(|| AppError::invalid("ACP conversation is disconnected"))
}

fn active() -> &'static Mutex<HashMap<String, ConversationHandle>> {
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}
