use std::sync::Arc;

use agent_client_protocol::schema::v1::{
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome,
};
use tokio_util::sync::CancellationToken;

use super::{authorize, PermissionPolicy};

fn request(options: serde_json::Value) -> RequestPermissionRequest {
    serde_json::from_value(serde_json::json!({
        "sessionId": "session", "toolCall": {"toolCallId": "call"}, "options": options,
    }))
    .unwrap()
}

#[tokio::test]
async fn trusted_policy_grants_allow_always_only_when_explicitly_opted_in() {
    let request = request(serde_json::json!([
        {"optionId": "reject", "name": "Reject", "kind": "reject_once"},
        {"optionId": "once", "name": "Allow", "kind": "allow_once"},
        {"optionId": "persistent", "name": "Always", "kind": "allow_always"}
    ]));
    let response = authorize(
        &request,
        &PermissionPolicy::Trusted,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    let RequestPermissionOutcome::Selected(selection) = response.outcome else {
        panic!("must authorize in trusted mode")
    };
    assert_eq!(selection.option_id.to_string(), "persistent");
}

#[tokio::test]
async fn deny_elevated_policy_rejects_requests_instead_of_auto_allowing() {
    let req = request(serde_json::json!([
        {"optionId": "reject", "name": "Reject", "kind": "reject_once"},
        {"optionId": "once", "name": "Allow", "kind": "allow_once"},
        {"optionId": "persistent", "name": "Always", "kind": "allow_always"}
    ]));
    let response = authorize(
        &req,
        &PermissionPolicy::DenyElevated,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    let RequestPermissionOutcome::Selected(selection) = response.outcome else {
        panic!("must select reject option")
    };
    assert_eq!(selection.option_id.to_string(), "reject");

    let no_reject = request(serde_json::json!([
        {"optionId": "once", "name": "Allow", "kind": "allow_once"}
    ]));
    let cancelled = authorize(
        &no_reject,
        &PermissionPolicy::DenyElevated,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert!(matches!(
        cancelled.outcome,
        RequestPermissionOutcome::Cancelled
    ));
}

#[tokio::test]
async fn interactive_policy_delegates_to_user_approver() {
    let req = request(serde_json::json!([
        {"optionId": "once", "name": "Allow once", "kind": "allow_once"},
        {"optionId": "reject", "name": "Reject", "kind": "reject_once"}
    ]));
    let policy = PermissionPolicy::Interactive(Arc::new(|incoming, _| {
        Box::pin(async move {
            let chosen = incoming.options[0].option_id.clone();
            Ok(RequestPermissionResponse::new(
                RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(chosen)),
            ))
        })
    }));
    let response = authorize(&req, &policy, &CancellationToken::new())
        .await
        .unwrap();
    let RequestPermissionOutcome::Selected(selection) = response.outcome else {
        panic!("must return interactive choice")
    };
    assert_eq!(selection.option_id.to_string(), "once");
}

#[tokio::test]
async fn trusted_policy_allows_once_option_and_rejects_missing_allow_option() {
    let once_request = request(serde_json::json!([
        {"optionId": "once", "name": "Allow", "kind": "allow_once"}
    ]));
    assert!(matches!(
        authorize(
            &once_request,
            &PermissionPolicy::Trusted,
            &CancellationToken::new()
        )
        .await
        .unwrap()
        .outcome,
        RequestPermissionOutcome::Selected(_)
    ));
    assert!(authorize(
        &request(serde_json::json!([])),
        &PermissionPolicy::Trusted,
        &CancellationToken::new()
    )
    .await
    .is_err());
}

#[tokio::test]
async fn cancellation_never_authorizes_a_new_operation() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let response = authorize(
        &request(serde_json::json!([])),
        &PermissionPolicy::Trusted,
        &cancellation,
    )
    .await
    .unwrap();
    assert!(matches!(
        response.outcome,
        RequestPermissionOutcome::Cancelled
    ));
}

#[test]
fn full_access_mode_is_only_requested_in_trusted_policy() {
    let response = serde_json::from_value(serde_json::json!({
        "sessionId": "session", "modes": {"currentModeId": "read-only", "availableModes": [
            {"id":"read-only", "name":"Read only"}, {"id":"agent-full-access", "name":"Full"}
        ]}
    }))
    .unwrap();
    assert!(super::trusted_mode_request(
        Some("agent-full-access"),
        &response,
        &PermissionPolicy::DenyElevated,
    )
    .unwrap()
    .is_none());
    let change = super::trusted_mode_request(
        Some("agent-full-access"),
        &response,
        &PermissionPolicy::Trusted,
    )
    .unwrap()
    .unwrap();
    assert_eq!(change.mode_id.to_string(), "agent-full-access");
    assert!(super::trusted_mode_request(
        Some("bypassPermissions"),
        &response,
        &PermissionPolicy::Trusted,
    )
    .is_err());
    assert!(
        super::trusted_mode_request(None, &response, &PermissionPolicy::Trusted)
            .unwrap()
            .is_none()
    );
    let mut current = response;
    current.modes.as_mut().unwrap().current_mode_id = "agent-full-access".into();
    assert!(super::trusted_mode_request(
        Some("agent-full-access"),
        &current,
        &PermissionPolicy::Trusted,
    )
    .unwrap()
    .is_none());
}

/// The launched adapters own these ids: `@agentclientprotocol/codex-acp` advertises
/// `read-only`, `workspace-write`, `agent`, `agent-full-access`, and
/// `@agentclientprotocol/claude-agent-acp` advertises Claude Code's permission modes. A session
/// dies with "full-access session mode is unavailable" the moment this vocabulary drifts.
#[test]
fn trusted_session_modes_match_the_launched_adapter_vocabulary() {
    use crate::ai::contracts::ExternalAgentKind;
    use crate::external_agents::registry::trusted_session_mode;

    assert_eq!(
        trusted_session_mode(ExternalAgentKind::Codex),
        Some("agent-full-access")
    );
    assert_eq!(
        trusted_session_mode(ExternalAgentKind::ClaudeCode),
        Some("bypassPermissions")
    );
    assert_eq!(trusted_session_mode(ExternalAgentKind::GeminiCli), None);
}
