use super::*;
use crate::extensions::contracts::ConnectorAuthorization;

fn definition() -> ConnectorDefinition {
    ConnectorDefinition {
        id: "service".into(),
        revision: "one".into(),
        name: "Service".into(),
        description: "Reference".into(),
        plugin_id: Some("plugin".into()),
        component_name: Some("component".into()),
        endpoint: None,
        authorization: ConnectorAuthorization::None,
    }
}
fn connection(id: &str, account: &str) -> McpConnection {
    McpConnection {
        id: id.into(),
        connector_id: "service".into(),
        name: id.into(),
        revision: "one".into(),
        account_id: Some(account.into()),
        enabled: false,
        tools: vec![],
    }
}

#[tokio::test]
async fn account_revocation_cancels_pending_inspection_and_task_scopes_together() {
    let sessions = Mutex::new(Sessions::new());
    let shutdown = CancellationToken::new();
    let connection = connection("connection", "account");
    let inspection = register(
        &sessions,
        &shutdown,
        "inspection",
        &connection,
        &definition(),
    )
    .await;
    let task = register(&sessions, &shutdown, "task", &connection, &definition()).await;
    let unrelated = register(
        &sessions,
        &shutdown,
        "unrelated",
        &self::connection("other", "other-account"),
        &definition(),
    )
    .await;
    revoke_matching(&sessions, |_, slot| {
        slot.connection.account_id.as_deref() == Some("account")
    })
    .await;
    assert!(inspection.cancellation.is_cancelled());
    assert!(task.cancellation.is_cancelled());
    assert!(!unrelated.cancellation.is_cancelled());
    assert_eq!(sessions.lock().await.len(), 1);
}

#[tokio::test]
async fn releasing_inspection_preserves_task_and_shutdown_cancels_remaining_scope() {
    let sessions = Mutex::new(Sessions::new());
    let shutdown = CancellationToken::new();
    let connection = connection("connection", "account");
    let inspection = register(
        &sessions,
        &shutdown,
        "inspection",
        &connection,
        &definition(),
    )
    .await;
    let task = register(&sessions, &shutdown, "task", &connection, &definition()).await;
    revoke_matching(&sessions, |(scope, _), _| scope == "inspection").await;
    assert!(inspection.cancellation.is_cancelled());
    assert!(!task.cancellation.is_cancelled());
    shutdown.cancel();
    assert!(task.cancellation.is_cancelled());
}
