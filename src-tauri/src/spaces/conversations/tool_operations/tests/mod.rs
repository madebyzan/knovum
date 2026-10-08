use crate::spaces::{conversations::execution::tests::fixture, db};
use serde_json::json;
async fn dispatched(pool: &sqlx::SqlitePool) {
    sqlx::query("INSERT INTO spaces_conversation_tool_operations(id,run_id,provider_call_id,wire_name,source,invocation_id,context,context_digest,read_only,approval,state,created_at) VALUES('operation','run','call','external', $1,'intent','{}','digest',0,'allow','dispatching',1)")
        .bind(json!({"type":"mcp_tool","connectionId":"connection","accountId":null,"toolName":"publish","schemaDigest":"schema"}).to_string()).execute(pool).await.unwrap();
}
#[tokio::test]
async fn restart_preserves_unknown_write_and_blocks_new_writes() {
    let pool = fixture::database().await;
    dispatched(&pool).await;
    super::recover(&pool).await.unwrap();
    assert!(super::require_reconciled(&pool, "run").await.is_err());
    let state: String = sqlx::query_scalar(
        "SELECT state FROM spaces_conversation_tool_operations WHERE id='operation'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state, "outcome_unknown");
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM spaces_conversation_messages WHERE role='tool'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    super::recover(&pool).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM spaces_conversation_messages WHERE role='tool'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    let mut tx = db::begin_write(&pool).await.unwrap();
    assert!(super::require_reconciled_on(&mut tx, "run").await.is_err());
}
#[tokio::test]
async fn late_receipt_records_fact_without_reviving_cancelled_run() {
    let pool = fixture::database().await;
    dispatched(&pool).await;
    sqlx::query("UPDATE spaces_runs SET state='cancelled',completed_at=2,cancel_requested_at=2 WHERE id='run'").execute(&pool).await.unwrap();
    super::receipts::complete(
        &pool,
        "operation",
        "completed",
        &json!({"content":[{"type":"text","text":"committed remotely"}]}),
    )
    .await
    .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM spaces_runs WHERE id='run'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "cancelled"
    );
    assert!(
        super::receipts::complete(&pool, "operation", "completed", &json!({}))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn explicit_retry_preserves_unknown_fact_and_uses_one_new_attempt() {
    use super::intervention::ToolOperationDecision;
    let pool = fixture::database().await;
    dispatched(&pool).await;
    super::recover(&pool).await.unwrap();
    super::decisions::apply(&pool, "operation", ToolOperationDecision::NewAttempt)
        .await
        .unwrap();
    let attempts: Vec<(i64, String, Option<String>, Option<String>)> = sqlx::query_as("SELECT attempt,state,resolution,approval FROM spaces_conversation_tool_operations ORDER BY attempt")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(
        attempts,
        vec![
            (
                1,
                "outcome_unknown".into(),
                Some("new_attempt".into()),
                Some("allow".into())
            ),
            (2, "prepared".into(), None, Some("allow".into()))
        ]
    );
    super::require_reconciled(&pool, "run").await.unwrap();
    assert_eq!(
        super::decisions::apply(&pool, "operation", ToolOperationDecision::NewAttempt)
            .await
            .unwrap_err(),
        "tool-outcome-intervention-stale"
    );
}
#[tokio::test]
async fn retry_respects_budget_and_only_records_authority_while_paused() {
    use super::intervention::ToolOperationDecision;
    let pool = fixture::database().await;
    dispatched(&pool).await;
    super::recover(&pool).await.unwrap();
    sqlx::query("UPDATE spaces_runs SET budget=json_set(budget,'$.maxTools',1) WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        super::decisions::apply(&pool, "operation", ToolOperationDecision::NewAttempt)
            .await
            .unwrap_err(),
        "conversation-tool-budget-exhausted"
    );
    sqlx::query("UPDATE spaces_runs SET state='paused',pause_requested_at=2,budget=json_set(budget,'$.maxTools',4) WHERE id='run'").execute(&pool).await.unwrap();
    super::decisions::apply(&pool, "operation", ToolOperationDecision::NewAttempt)
        .await
        .unwrap();
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state, "paused");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM spaces_conversation_tool_operations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}
#[tokio::test]
async fn denial_is_a_canonical_receipt_and_cannot_be_approved_later() {
    use super::intervention::ToolOperationDecision;
    let pool = fixture::database().await;
    dispatched(&pool).await;
    sqlx::query("UPDATE spaces_conversation_tool_operations SET state='prepared',approval=NULL")
        .execute(&pool)
        .await
        .unwrap();
    super::decisions::apply(&pool, "operation", ToolOperationDecision::Deny)
        .await
        .unwrap();
    let (state,approval,receipt):(String,String,String)=sqlx::query_as("SELECT state,approval,output FROM spaces_conversation_tool_operations WHERE id='operation'").fetch_one(&pool).await.unwrap();
    assert_eq!(state, "failed");
    assert_eq!(approval, "deny");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&receipt).unwrap()["isError"],
        true
    );
    assert_eq!(
        super::decisions::apply(&pool, "operation", ToolOperationDecision::Approve)
            .await
            .unwrap_err(),
        "tool-approval-stale"
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM spaces_conversation_messages WHERE role='tool'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn restart_records_uncertainty_without_resuming_a_paused_run() {
    let pool = fixture::database().await;
    dispatched(&pool).await;
    sqlx::query("UPDATE spaces_runs SET state='paused',pause_requested_at=2 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    super::recover(&pool).await.unwrap();
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state, "paused");
    assert!(super::require_reconciled(&pool, "run").await.is_err());
}
