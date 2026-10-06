use super::fixture;
use crate::agent_runtime::protocol::{
    messages::{
        AssistantPlanItem, AssistantPlanState, AssistantProjectedMessage, AssistantProjection,
    },
    models::AssistantModelCheckpoint,
};

fn projection() -> AssistantProjection {
    AssistantProjection {
        thread_id: "thread".into(),
        checkpoint_id: "checkpoint".into(),
        messages: vec![AssistantProjectedMessage {
            message_id: "answer".into(),
            text: "Complete answer".into(),
        }],
        accepted_input_ids: vec!["input:task".into()],
        model_checkpoints: vec![AssistantModelCheckpoint {
            graph_namespace: "".into(),
            message_ids: vec!["answer".into()],
            request_ids: vec!["summary".into()],
        }],
        plan: vec![AssistantPlanItem {
            content: "Read evidence".into(),
            state: AssistantPlanState::Completed,
        }],
    }
}

#[tokio::test]
async fn replayed_projection_commits_one_answer_plan_and_input_acknowledgement() {
    let pool = fixture::database().await;
    sqlx::query("INSERT INTO spaces_assistant_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,estimated_input_tokens,max_output_tokens,state,created_at)
        VALUES('summary','task','context_summary','','before',10,10,'received',1)")
        .execute(&pool).await.unwrap();
    for _ in 0..2 {
        super::super::projection::commit(&pool, &fixture::ids(), projection())
            .await
            .unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_assistant_messages")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM spaces_run_events WHERE kind='assistant_plan_updated'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT input_accepted FROM spaces_assistant_tasks")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM spaces_assistant_model_calls")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "committed"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT projection_checkpoint_id FROM spaces_assistant_execution_segments"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        "checkpoint"
    );
}

#[tokio::test]
async fn projection_rejects_a_different_thread_without_mutating_history() {
    let pool = fixture::database().await;
    let mut value = projection();
    value.thread_id = "unbound".into();
    assert!(
        super::super::projection::commit(&pool, &fixture::ids(), value)
            .await
            .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_assistant_messages")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}
