use std::sync::Arc;

use super::{pending_for_session, request_decision};
use crate::external_agents::session::AgentUpdateHandler;
use agent_client_protocol::schema::v1::RequestPermissionRequest;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn dropping_permission_future_removes_pending_request() {
    let request: RequestPermissionRequest = serde_json::from_value(serde_json::json!({
        "sessionId": "session",
        "toolCall": { "toolCallId": "call", "title": "Inspect file" },
        "options": [{ "optionId": "allow", "name": "Allow", "kind": "allow_once" }]
    }))
    .expect("permission request");
    let cancellation = CancellationToken::new();
    let on_update: AgentUpdateHandler = Arc::new(|_| Ok(()));
    let session = uuid::Uuid::new_v4().to_string();
    let mut future = Box::pin(request_decision(
        &session,
        &request,
        &cancellation,
        &on_update,
    ));
    tokio::select! {
        biased;
        _ = &mut future => panic!("request should remain pending"),
        _ = futures::future::ready(()) => {},
    }
    assert_eq!(pending_for_session(&session).expect("pending").len(), 1);
    drop(future);
    assert!(pending_for_session(&session).expect("released").is_empty());
}
