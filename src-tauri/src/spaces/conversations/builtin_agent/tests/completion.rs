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
