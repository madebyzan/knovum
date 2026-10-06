use super::fixture;
use crate::{
    agent_runtime::protocol::models::{
        AssistantModelAdmission, AssistantModelCompletion, AssistantModelFailure,
        AssistantModelPurpose,
    },
    spaces::assistant::execution::ledger,
};

fn admission(id: &str) -> AssistantModelAdmission {
    AssistantModelAdmission {
        identity: fixture::identity(),
        request_id: id.into(),
        purpose: AssistantModelPurpose::ContextSummary,
        checkpoint_id: Some("checkpoint".into()),
        estimated_input_tokens: 100,
        max_output_tokens: 100,
        graph_namespace: "model:1".into(),
    }
}

fn completion(id: &str, input: Option<u64>) -> AssistantModelCompletion {
    AssistantModelCompletion {
        identity: fixture::identity(),
        request_id: id.into(),
        input_tokens: input,
        output_tokens: Some(10),
        message_id: Some(format!("message:{id}")),
    }
}

#[tokio::test]
async fn unknown_calls_block_retries_and_authorization_does_not_erase_reserved_cost() {
    let pool = fixture::database().await;
    ledger::admit(&pool, "task", admission("first"))
        .await
        .unwrap();
    ledger::fail(
        &pool,
        "task",
        AssistantModelFailure {
            identity: fixture::identity(),
            request_id: "first".into(),
        },
    )
    .await
    .unwrap();
    assert!(ledger::admit(&pool, "task", admission("second"))
        .await
        .is_err());
    sqlx::query("UPDATE spaces_assistant_model_calls SET retry_authorized_at=1 WHERE id='first'")
        .execute(&pool)
        .await
        .unwrap();
    ledger::admit(&pool, "task", admission("second"))
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_runs SET total_tokens=700 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(ledger::admit(&pool, "task", admission("third"))
        .await
        .is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_assistant_model_calls")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );
}

#[tokio::test]
async fn late_known_usage_survives_cancellation_and_is_counted_once() {
    let pool = fixture::database().await;
    ledger::admit(&pool, "task", admission("first"))
        .await
        .unwrap();
    crate::spaces::runs::request_cancellation(&pool, "run")
        .await
        .unwrap();
    ledger::complete(&pool, "task", completion("first", Some(1200)))
        .await
        .unwrap();
    assert!(
        ledger::complete(&pool, "task", completion("first", Some(1200)))
            .await
            .is_err()
    );
    assert_eq!(
        crate::spaces::runs::load_run(&pool, "run")
            .await
            .unwrap()
            .total_tokens,
        1210
    );
    assert!(ledger::admit(&pool, "task", admission("second"))
        .await
        .is_err());
}

#[tokio::test]
async fn received_summary_allows_the_same_live_node_to_continue_but_not_crash_replay() {
    let pool = fixture::database().await;
    ledger::admit(&pool, "task", admission("summary"))
        .await
        .unwrap();
    ledger::complete(&pool, "task", completion("summary", Some(20)))
        .await
        .unwrap();
    ledger::admit(&pool, "task", admission("main"))
        .await
        .unwrap();
    assert!(ledger::require_reconciled(&pool, "task").await.is_err());
}

#[tokio::test]
async fn missing_usage_is_never_zero_and_blocks_further_paid_calls() {
    let pool = fixture::database().await;
    ledger::admit(&pool, "task", admission("first"))
        .await
        .unwrap();
    ledger::complete(&pool, "task", completion("first", None))
        .await
        .unwrap();
    assert!(
        ledger::admit_tool(&pool, "task", "run", "operation", "read_file")
            .await
            .is_err()
    );
    assert!(ledger::admit(&pool, "task", admission("second"))
        .await
        .is_err());
    assert!(ledger::require_reconciled(&pool, "task").await.is_err());
}

#[tokio::test]
async fn replayed_tool_identity_consumes_one_admission_and_is_independent_of_epoch() {
    let pool = fixture::database().await;
    for _ in 0..2 {
        ledger::admit_tool(&pool, "task", "run", "operation", "read_file")
            .await
            .unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_assistant_tool_admissions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    let identity = fixture::identity();
    let first = super::super::gateway::tool_key(
        &identity,
        "tools:stable",
        "message",
        "call",
        "knovum_node_create_v1",
    );
    let mut resumed = identity;
    resumed.control_epoch = 7;
    assert_eq!(
        first,
        super::super::gateway::tool_key(
            &resumed,
            "tools:stable",
            "message",
            "call",
            "knovum_node_create_v1"
        )
    );
    assert_ne!(
        first,
        super::super::gateway::tool_key(
            &resumed,
            "tools:other",
            "message",
            "call",
            "knovum_node_create_v1"
        )
    );
}

#[tokio::test]
async fn paid_usage_overrun_prevents_new_tools_and_model_calls() {
    let pool = fixture::database().await;
    ledger::admit(&pool, "task", admission("first"))
        .await
        .unwrap();
    ledger::complete(&pool, "task", completion("first", Some(1200)))
        .await
        .unwrap();
    assert!(
        ledger::admit_tool(&pool, "task", "run", "operation", "read_file")
            .await
            .is_err()
    );
    assert!(ledger::admit(&pool, "task", admission("second"))
        .await
        .is_err());
    assert_eq!(
        crate::spaces::runs::load_run(&pool, "run")
            .await
            .unwrap()
            .total_tokens,
        1210
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_assistant_tool_admissions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}
