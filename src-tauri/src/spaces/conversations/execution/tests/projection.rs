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
    sqlx::query("INSERT INTO spaces_builtin_agent_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,state,created_at)
        VALUES('summary','task','context_summary','','before','received',1)")
        .execute(&pool).await.unwrap();
    sqlx::query("CREATE TABLE projection_updates(kind TEXT); CREATE TRIGGER input_acknowledged AFTER UPDATE OF input_accepted ON spaces_builtin_agent_tasks BEGIN INSERT INTO projection_updates VALUES('input'); END; CREATE TRIGGER model_committed AFTER UPDATE OF state ON spaces_builtin_agent_model_calls BEGIN INSERT INTO projection_updates VALUES('model'); END;")
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
    let updates: Vec<String> = sqlx::query_scalar("SELECT kind FROM projection_updates")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(updates, ["input", "model"]);
}

#[tokio::test]
async fn settles_many_receipts_without_crossing_graph_namespaces() {
    let pool = fixture::database().await;
    let ids = (0..300)
        .map(|index| format!("receipt-{index}"))
        .collect::<Vec<_>>();
    sqlx::query("INSERT INTO spaces_builtin_agent_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,state,created_at) SELECT value,'task','context_summary','','before','received',1 FROM json_each($1)")
        .bind(serde_json::to_string(&ids).unwrap()).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_builtin_agent_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,state,created_at) VALUES('child-receipt','task','context_summary','child','before','received',1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_builtin_agent_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,message_id,state,created_at) VALUES('message-receipt','task','main','','before','answer','received',1)")
        .execute(&pool).await.unwrap();
    let mut value = projection();
    value.model_checkpoints[0].request_ids = ids;
    value.model_checkpoints[0]
        .request_ids
        .push("child-receipt".into());
    super::super::projection::commit(&pool, &fixture::ids(), value)
        .await
        .unwrap();
    let committed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_builtin_agent_model_calls WHERE state='committed'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(committed, 301);
    let child: String = sqlx::query_scalar(
        "SELECT state FROM spaces_builtin_agent_model_calls WHERE id='child-receipt'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(child, "received");
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
