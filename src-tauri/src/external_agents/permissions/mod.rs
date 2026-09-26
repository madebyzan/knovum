use crate::command_error::CommandError;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use agent_client_protocol::schema::v1::{
    PermissionOptionKind, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome,
};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use super::session::AgentUpdateHandler;
use super::updates::{AgentPermissionOption, AgentSessionUpdate};

struct PendingDecision {
    title: String,
    options: Vec<super::updates::AgentPermissionOption>,
    option_ids: Vec<String>,
    sender: oneshot::Sender<Option<String>>,
}

pub(crate) struct PendingAgentPermission {
    pub request_id: String,
    pub title: String,
    pub options: Vec<super::updates::AgentPermissionOption>,
}

pub(crate) fn pending_for_session(session_id: &str) -> Result<Vec<PendingAgentPermission>, String> {
    let decisions = pending()
        .lock()
        .map_err(|_| "permission-broker-unavailable")?;
    Ok(decisions
        .iter()
        .filter(|((id, _), _)| id == session_id)
        .map(|((_, request_id), decision)| PendingAgentPermission {
            request_id: request_id.clone(),
            title: decision.title.clone(),
            options: decision.options.clone(),
        })
        .collect())
}

static PENDING: OnceLock<Mutex<HashMap<(String, String), PendingDecision>>> = OnceLock::new();

pub(super) async fn request_decision(
    task_id: &str,
    request: &RequestPermissionRequest,
    cancellation: &CancellationToken,
    on_update: &AgentUpdateHandler,
) -> RequestPermissionResponse {
    let request_id = uuid::Uuid::new_v4().to_string();
    let options = request
        .options
        .iter()
        .map(|option| AgentPermissionOption {
            id: option.option_id.to_string(),
            label: option.name.clone(),
            kind: option_kind(option.kind),
        })
        .collect::<Vec<_>>();
    let option_ids = options.iter().map(|option| option.id.clone()).collect();
    let (sender, receiver) = oneshot::channel();
    {
        let Ok(mut decisions) = pending().lock() else {
            log::error!("permission broker unavailable");
            return cancelled();
        };
        decisions.insert(
            (task_id.to_owned(), request_id.clone()),
            PendingDecision {
                title: request
                    .tool_call
                    .fields
                    .title
                    .clone()
                    .unwrap_or_else(|| "External Agent operation".to_owned()),
                options: options.clone(),
                option_ids,
                sender,
            },
        );
    }
    let _lease = DecisionLease {
        session_id: task_id.to_owned(),
        request_id: request_id.clone(),
    };
    let update = AgentSessionUpdate::PermissionRequested {
        request_id: request_id.clone(),
        title: request
            .tool_call
            .fields
            .title
            .clone()
            .unwrap_or_else(|| "External Agent operation".to_owned()),
        options,
    };
    if on_update(update).is_err() {
        return cancelled();
    }
    let selection = tokio::select! {
        value = receiver => value.ok().flatten(),
        _ = cancellation.cancelled() => None,
    };
    selection.map_or_else(cancelled, selected)
}

#[tauri::command]
pub fn decide_external_agent_permission(
    session_id: String,
    request_id: String,
    option_id: Option<String>,
) -> Result<(), CommandError> {
    let key = (session_id, request_id);
    let mut pending = pending()
        .lock()
        .map_err(|_| "permission-broker-unavailable")?;
    let decision = pending.get(&key).ok_or("permission-request-unavailable")?;
    if option_id
        .as_ref()
        .is_some_and(|id| !decision.option_ids.contains(id))
    {
        return Err("permission-option-invalid".into());
    }
    let decision = pending
        .remove(&key)
        .ok_or("permission-request-unavailable")?;
    drop(pending);
    Ok((decision
        .sender
        .send(option_id)
        .map_err(|_| "permission-request-closed"))?)
}

fn pending() -> &'static Mutex<HashMap<(String, String), PendingDecision>> {
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

fn remove(task_id: &str, request_id: &str) {
    if let Ok(mut guard) = pending().lock() {
        guard.remove(&(task_id.to_owned(), request_id.to_owned()));
    }
}

struct DecisionLease {
    session_id: String,
    request_id: String,
}

impl Drop for DecisionLease {
    fn drop(&mut self) {
        remove(&self.session_id, &self.request_id);
    }
}

fn selected(option_id: String) -> RequestPermissionResponse {
    RequestPermissionResponse::new(RequestPermissionOutcome::Selected(
        SelectedPermissionOutcome::new(option_id),
    ))
}

fn cancelled() -> RequestPermissionResponse {
    RequestPermissionResponse::new(RequestPermissionOutcome::Cancelled)
}

fn option_kind(kind: PermissionOptionKind) -> &'static str {
    match kind {
        PermissionOptionKind::AllowOnce => "allow_once",
        PermissionOptionKind::AllowAlways => "allow_always",
        PermissionOptionKind::RejectOnce => "reject_once",
        PermissionOptionKind::RejectAlways => "reject_always",
        _ => "reject_once",
    }
}

#[cfg(test)]
mod tests;
