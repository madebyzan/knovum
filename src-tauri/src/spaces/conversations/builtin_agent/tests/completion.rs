use crate::spaces::conversations::{
    builtin_agent::completion, execution::tests::fixture, ConversationTaskFinishedState,
};

#[tokio::test]
async fn an_unsettled_child_prevents_success_and_failed_children_remain_visible() {
    let pool = fixture::database().await;
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
        VALUES('child','space','run','builtin_agent','task','action','running','{}','{}','{}',1,1)")
        .execute(&pool).await.unwrap();
    assert!(completion::complete(&pool, &fixture::ids()).await.is_err());
    assert_eq!(
        crate::spaces::runs::load_run(&pool, "run")
            .await
            .unwrap()
            .state,
        crate::spaces::runs::RunState::Running
    );
    sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=2 WHERE id='child'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        completion::complete(&pool, &fixture::ids()).await.unwrap(),
        ConversationTaskFinishedState::CompletedWithErrors
    ));
    assert_eq!(
        crate::spaces::runs::load_run(&pool, "run")
            .await
            .unwrap()
            .state,
        crate::spaces::runs::RunState::CompletedWithErrors
    );
}

#[tokio::test]
async fn cancellation_prevents_final_publication() {
    let pool = fixture::database().await;
    crate::spaces::runs::request_cancellation(&pool, "run")
        .await
        .unwrap();
    assert!(completion::complete(&pool, &fixture::ids()).await.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM spaces_conversation_messages WHERE role='assistant'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}

#[tokio::test]
async fn paid_usage_at_the_limit_can_complete_but_an_overrun_cannot_succeed() {
    for tokens in [1_000, 1_001] {
        let pool = fixture::database().await;
        sqlx::query("UPDATE spaces_runs SET input_tokens=$1,total_tokens=$1 WHERE id='run'")
            .bind(tokens)
            .execute(&pool)
            .await
            .unwrap();
        let outcome = completion::complete(&pool, &fixture::ids()).await;
        let expected = if tokens == 1_000 {
            assert!(matches!(
                outcome.unwrap(),
                ConversationTaskFinishedState::Succeeded
            ));
            crate::spaces::runs::RunState::Succeeded
        } else {
            let error = outcome.unwrap_err();
            assert!(error.starts_with("budget-exhausted"));
            completion::fail(&pool, "run", &error).await.unwrap();
            crate::spaces::runs::RunState::Failed
        };
        let run = crate::spaces::runs::load_run(&pool, "run").await.unwrap();
        assert_eq!(run.state, expected);
        assert_eq!(run.total_tokens, tokens);
    }
}

#[tokio::test]
async fn failure_settlement_preserves_write_and_state_read_failures() {
    let pool = fixture::database().await;
    sqlx::query("CREATE TRIGGER reject_run_update BEFORE UPDATE ON spaces_runs BEGIN SELECT RAISE(ABORT,'write rejected'); END")
        .execute(&pool).await.unwrap();

    let (state, error) = completion::settle_failure(&pool, "run", "driver failed").await;
    assert!(matches!(
        state,
        ConversationTaskFinishedState::SettlementFailed
    ));
    let error = error.unwrap();
    assert!(error.contains("driver failed"));
    assert!(error.contains("write rejected"));
    assert!(error.contains("remained Running"));

    let closed = fixture::database().await;
    closed.close().await;
    let (state, error) = completion::settle_failure(&closed, "run", "transport failed").await;
    assert!(matches!(
        state,
        ConversationTaskFinishedState::SettlementFailed
    ));
    let error = error.unwrap();
    assert!(error.contains("transport failed"));
    assert!(error.contains("failure could not be recorded"));
    assert!(error.contains("final state could not be read"));
}

#[tokio::test]
async fn confirmed_run_states_are_authoritative_during_failure_settlement() {
    for (state, completed, control, expected) in [
        (
            "waiting",
            None,
            None,
            ConversationTaskFinishedState::Waiting,
        ),
        (
            "paused",
            None,
            Some("pause"),
            ConversationTaskFinishedState::Paused,
        ),
        (
            "failed",
            Some(2),
            None,
            ConversationTaskFinishedState::Failed,
        ),
        (
            "completed_with_errors",
            Some(2),
            None,
            ConversationTaskFinishedState::CompletedWithErrors,
        ),
        (
            "succeeded",
            Some(2),
            None,
            ConversationTaskFinishedState::Succeeded,
        ),
        (
            "cancelled",
            Some(2),
            Some("cancel"),
            ConversationTaskFinishedState::Cancelled,
        ),
        (
            "running",
            None,
            Some("pause"),
            ConversationTaskFinishedState::Paused,
        ),
        (
            "running",
            None,
            Some("cancel"),
            ConversationTaskFinishedState::Cancelled,
        ),
    ] {
        let pool = fixture::database().await;
        sqlx::query(
            "UPDATE spaces_runs SET state=$1,completed_at=$2,pause_requested_at=$3,cancel_requested_at=$4 WHERE id='run'",
        )
        .bind(state)
        .bind(completed)
        .bind(control.filter(|value| *value == "pause").map(|_| 2_i64))
        .bind(control.filter(|value| *value == "cancel").map(|_| 2_i64))
        .execute(&pool)
        .await
        .unwrap();

        let (actual, error) = completion::settle_failure(&pool, "run", "late failure").await;
        assert_eq!(actual, expected);
        assert!(error.is_none());
    }
}

#[tokio::test]
async fn uncontrolled_running_state_is_not_reported_as_failed() {
    let pool = fixture::database().await;
    let error = completion::stopped_state(&pool, "run").await.unwrap_err();
    assert!(error.contains("remained Running"));
}
