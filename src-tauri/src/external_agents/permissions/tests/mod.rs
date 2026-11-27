use agent_client_protocol::schema::v1::{RequestPermissionOutcome, RequestPermissionRequest};
use tokio_util::sync::CancellationToken;

use super::authorize;

fn request(options: serde_json::Value) -> RequestPermissionRequest {
    serde_json::from_value(serde_json::json!({
        "sessionId": "session", "toolCall": {"toolCallId": "call"}, "options": options,
    }))
    .unwrap()
}

#[test]
fn grants_runtime_allow_always_without_a_user_decision() {
    let request = request(serde_json::json!([
        {"optionId": "reject", "name": "Reject", "kind": "reject_once"},
        {"optionId": "once", "name": "Allow", "kind": "allow_once"},
        {"optionId": "persistent", "name": "Always", "kind": "allow_always"}
    ]));
    let response = authorize(&request, &CancellationToken::new()).unwrap();
    let RequestPermissionOutcome::Selected(selection) = response.outcome else {
        panic!("must authorize")
    };
    assert_eq!(selection.option_id.to_string(), "persistent");
}

#[test]
fn allows_runtime_once_option_and_rejects_missing_allow_option() {
    let once_request = request(serde_json::json!([
        {"optionId": "once", "name": "Allow", "kind": "allow_once"}
    ]));
    assert!(matches!(
        authorize(&once_request, &CancellationToken::new())
            .unwrap()
            .outcome,
        RequestPermissionOutcome::Selected(_)
    ));
    assert!(authorize(&request(serde_json::json!([])), &CancellationToken::new()).is_err());
}

#[test]
fn cancellation_never_authorizes_a_new_operation() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let response = authorize(&request(serde_json::json!([])), &cancellation).unwrap();
    assert!(matches!(
        response.outcome,
        RequestPermissionOutcome::Cancelled
    ));
}

#[test]
fn full_access_mode_must_be_advertised_and_never_silently_substituted() {
    let response = serde_json::from_value(serde_json::json!({
        "sessionId": "session", "modes": {"currentModeId": "read-only", "availableModes": [
            {"id":"read-only", "name":"Read only"}, {"id":"agent-full-access", "name":"Full"}
        ]}
    }))
    .unwrap();
    let change = super::trusted_mode_request(Some("agent-full-access"), &response)
        .unwrap()
        .unwrap();
    assert_eq!(change.mode_id.to_string(), "agent-full-access");
    assert!(super::trusted_mode_request(Some("bypassPermissions"), &response).is_err());
    assert!(super::trusted_mode_request(None, &response)
        .unwrap()
        .is_none());
    let mut current = response;
    current.modes.as_mut().unwrap().current_mode_id = "agent-full-access".into();
    assert!(
        super::trusted_mode_request(Some("agent-full-access"), &current)
            .unwrap()
            .is_none()
    );
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
