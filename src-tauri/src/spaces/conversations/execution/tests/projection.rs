use super::fixture;
use crate::agent_runtime::protocol::{
    messages::{AgentPlanItem, AgentPlanState, AgentProjectedMessage, AgentProjection},
    models::AgentModelCheckpoint,
};

fn projection() -> AgentProjection {
    AgentProjection {
        thread_id: "thread".into(),
        checkpoint_id: "checkpoint".into(),
        messages: vec![AgentProjectedMessage {
            message_id: "answer".into(),
            text: "Complete answer".into(),
        }],
        accepted_input_ids: vec!["input:task".into()],
        model_checkpoints: vec![AgentModelCheckpoint {
            graph_namespace: "".into(),
            message_ids: vec!["answer".into()],
            request_ids: vec!["summary".into()],
        }],
        plan: vec![AgentPlanItem {
            content: "Read evidence".into(),
            state: AgentPlanState::Completed,
        }],
    }
}

#[tokio::test]
async fn replayed_projection_commits_one_answer_plan_and_input_acknowledgement() {
    let pool = fixture::database().await;
    sqlx::query("INSERT INTO spaces_builtin_agent_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,estimated_input_tokens,max_output_tokens,state,created_at)
        VALUES('summary','task','context_summary','','before',10,10,'received',1)")
        .execute(&pool).await.unwrap();
    for _ in 0..2 {
        super::super::projection::commit(&pool, &fixture::ids(), projection())
            .await
            .unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_conversation_messages")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM spaces_run_events WHERE kind='builtin_agent_plan_updated'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT input_accepted FROM spaces_builtin_agent_tasks")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM spaces_builtin_agent_model_calls")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "committed"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT projection_checkpoint_id FROM spaces_builtin_agent_execution_segments"
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
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_conversation_messages")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}
