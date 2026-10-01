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
    let cancellation = active()
        .lock()
        .map_err(|_| "ACP conversation registry unavailable")?
        .get(id)
        .map(|handle| handle.cancellation.clone())
        .ok_or("ACP conversation is disconnected")?;
    cancellation.cancel();
    wait_until_inactive(id).await
}

pub async fn finish(id: &str) -> Result<(), String> {
    sender(id)
        .map_err(|error| error.to_string())?
        .send(ConversationRequest::Finish)
        .await
        .map_err(|_| "ACP conversation channel closed".to_owned())?;
    wait_until_inactive(id).await
}

async fn wait_until_inactive(id: &str) -> Result<(), String> {
    tokio::time::timeout(super::super::connection::CANCEL_TIMEOUT, async {
        while is_active(id) {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .map_err(|_| "ACP conversation did not close before the deadline")?;
    Ok(())
}

pub fn is_active(id: &str) -> bool {
    active().lock().is_ok_and(|items| items.contains_key(id))
}

pub(super) fn remove(id: &str) {
    if let Ok(mut conversations) = active().lock() {
        conversations.remove(id);
    }
}

fn sender(id: &str) -> AppResult<mpsc::Sender<ConversationRequest>> {
    let conversations = active()
        .lock()
        .map_err(|_| AppError::internal("ACP conversation registry unavailable"))?;
    conversations
        .get(id)
        .map(|handle| handle.requests.clone())
        .ok_or_else(|| AppError::invalid("ACP conversation is disconnected"))
}

fn active() -> &'static Mutex<HashMap<String, ConversationHandle>> {
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}
