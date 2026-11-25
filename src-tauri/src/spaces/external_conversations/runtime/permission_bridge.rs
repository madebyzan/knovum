use std::sync::{Arc, Mutex, PoisonError};

use agent_client_protocol::schema::v1::{
    PermissionOptionKind, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome,
};
use rmcp::model::{ElicitRequestParams, ElicitationAction};
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    extensions::mcp::McpInteraction,
    external_agents::permissions::{select_reject_or_cancel, PermissionApprover, PermissionPolicy},
    spaces::conversations::tool_operations::interactions::RunMcpInteraction,
};

/// Tracks which conversation Run currently owns a pooled Desktop Agent runtime so ACP permission
/// requests can surface in that Run's approval queue and never leak across turns.
#[derive(Default)]
pub(super) struct ActiveTurnRun(Mutex<Option<(SqlitePool, String)>>);

impl ActiveTurnRun {
    pub(super) fn bind(&self, pool: SqlitePool, run_id: String) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some((pool, run_id));
    }

    pub(super) fn clear(&self) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }

    fn current(&self) -> Option<(SqlitePool, String)> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

pub(super) fn interactive_policy(active_run: Arc<ActiveTurnRun>) -> PermissionPolicy {
    let approver: PermissionApprover = Arc::new(move |request, cancellation| {
        let active_run = active_run.clone();
        Box::pin(async move {
            let Some((pool, run_id)) = active_run.current() else {
                return Ok(select_reject_or_cancel(&request));
            };
            let Some(params) = build_elicitation_params(&request) else {
                return Ok(select_reject_or_cancel(&request));
            };
            let interaction = RunMcpInteraction { pool, run_id };
            match interaction.elicit(params, cancellation).await {
                Ok(result) => Ok(map_elicitation_result(&request, result)),
                Err(error) => {
                    log::warn!("Desktop Agent permission approval failed: {error}");
                    Ok(select_reject_or_cancel(&request))
                }
            }
        })
    });
    PermissionPolicy::Interactive(approver)
}

fn build_elicitation_params(request: &RequestPermissionRequest) -> Option<ElicitRequestParams> {
    if request.options.is_empty() {
        return None;
    }
    let option_ids: Vec<String> = request
        .options
        .iter()
        .map(|option| option.option_id.0.to_string())
        .collect();
    let option_names: Vec<String> = request
        .options
        .iter()
        .map(|option| option.name.clone())
        .collect();
    let default_id = request
        .options
        .iter()
        .find(|option| option.kind == PermissionOptionKind::AllowOnce)
        .or_else(|| {
            request
                .options
                .iter()
                .find(|option| option.kind == PermissionOptionKind::RejectOnce)
        })
        .unwrap_or(&request.options[0])
        .option_id
        .0
        .to_string();
    let tool_summary = request
        .tool_call
        .fields
        .title
        .as_deref()
        .filter(|title| !title.trim().is_empty())
        .unwrap_or("External Desktop Agent action");
    let message = format!(
        "Desktop Agent requests permission to proceed: {tool_summary}. Choose how to respond or decline."
    );
    serde_json::from_value(json!({
        "mode": "form",
        "message": message,
        "requestedSchema": {
            "type": "object",
            "properties": {
                "decision": {
                    "type": "string",
                    "title": "Decision",
                    "enum": option_ids,
                    "enumNames": option_names,
                    "default": default_id
                }
            },
            "required": ["decision"]
        }
    }))
    .ok()
}

fn map_elicitation_result(
    request: &RequestPermissionRequest,
    result: rmcp::model::ElicitResult,
) -> RequestPermissionResponse {
    if result.action != ElicitationAction::Accept {
        return select_reject_or_cancel(request);
    }
    let chosen_id = result
        .content
        .as_ref()
        .and_then(|value| value.get("decision"))
        .and_then(|value| value.as_str());
    let Some(chosen_id) = chosen_id else {
        return select_reject_or_cancel(request);
    };
    let Some(option) = request
        .options
        .iter()
        .find(|option| option.option_id.0.as_ref() == chosen_id)
    else {
        return select_reject_or_cancel(request);
    };
    RequestPermissionResponse::new(RequestPermissionOutcome::Selected(
        SelectedPermissionOutcome::new(option.option_id.clone()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ElicitResult;

    fn sample_request() -> RequestPermissionRequest {
        serde_json::from_value(json!({
            "sessionId": "session",
            "toolCall": {"toolCallId": "call", "title": "Run shell command: ls"},
            "options": [
                {"optionId": "once", "name": "Allow once", "kind": "allow_once"},
                {"optionId": "reject", "name": "Reject", "kind": "reject_once"}
            ]
        }))
        .unwrap()
    }

    #[test]
    fn builds_valid_elicitation_form_and_maps_accept_and_decline() {
        let req = sample_request();
        let params = build_elicitation_params(&req).expect("form params");
        let serialized = serde_json::to_value(&params).unwrap();
        assert_eq!(serialized["mode"], "form");
        assert!(serialized["message"]
            .as_str()
            .unwrap()
            .contains("Run shell command: ls"));

        let mut accept = ElicitResult::new(ElicitationAction::Accept);
        accept.content = Some(json!({"decision": "once"}));
        let accepted = map_elicitation_result(&req, accept);
        let RequestPermissionOutcome::Selected(selected) = accepted.outcome else {
            panic!("expected selected outcome");
        };
        assert_eq!(selected.option_id.0.as_ref(), "once");

        let declined = map_elicitation_result(&req, ElicitResult::new(ElicitationAction::Decline));
        let RequestPermissionOutcome::Selected(selected) = declined.outcome else {
            panic!("expected reject option on decline");
        };
        assert_eq!(selected.option_id.0.as_ref(), "reject");
    }
}
