use crate::spaces::assistant::{execution::tests::fixture, task::recovery};

#[tokio::test]
async fn resume_requires_a_durable_pause_and_cannot_queue_over_another_active_run() {
    let pool = fixture::database().await;
    sqlx::query("UPDATE spaces_runs SET state='paused',pause_requested_at=2 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(recovery::prepare(&pool, "task", "run", true, None)
        .await
        .is_err());
    sqlx::query("INSERT INTO spaces_run_events(run_id,sequence,kind,payload,created_at) VALUES('run',1,'assistant_paused_checkpoint','{}',3)")
        .execute(&pool).await.unwrap();
    sqlx::raw_sql("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
        VALUES('other','space','assistant','other','assistant','running','{}','{}','{}',1,1);
        INSERT INTO spaces_assistant_tasks(id,run_id,conversation_id,branch_id,grant_id,model_binding,catalog_digest,segment_id,input_id,created_at,updated_at)
        VALUES('other','other','conversation','branch','grant','{}','aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa','thread','input:other',1,1);")
        .execute(&pool).await.unwrap();
    assert!(recovery::prepare(&pool, "task", "run", true, None)
        .await
        .unwrap_err()
        .starts_with("assistant-busy"));
    assert_eq!(
        crate::spaces::runs::load_run(&pool, "run")
            .await
            .unwrap()
            .state,
        crate::spaces::runs::RunState::Paused
    );
    sqlx::query("UPDATE spaces_runs SET state='succeeded',completed_at=4 WHERE id='other'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        recovery::prepare(&pool, "task", "run", true, None)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        crate::spaces::runs::load_run(&pool, "run")
            .await
            .unwrap()
            .state,
        crate::spaces::runs::RunState::Running
    );
}

#[tokio::test]
async fn new_paid_attempt_requires_the_observed_version_and_keeps_the_uncertain_reservation() {
    let pool = fixture::database().await;
    sqlx::raw_sql("UPDATE spaces_runs SET state='waiting',wait_reason='model_outcome_unknown',state_version=2 WHERE id='run';
        INSERT INTO spaces_assistant_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,estimated_input_tokens,max_output_tokens,state,created_at)
        VALUES('lost','task','main','','checkpoint',100,100,'outcome_unknown',1);")
        .execute(&pool).await.unwrap();
    assert!(recovery::prepare(&pool, "task", "run", false, Some(1))
        .await
        .is_err());
    let authorized: Option<i64> = sqlx::query_scalar(
        "SELECT retry_authorized_at FROM spaces_assistant_model_calls WHERE id='lost'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(authorized.is_none());
    assert_eq!(
        recovery::prepare(&pool, "task", "run", false, Some(2))
            .await
            .unwrap(),
        1
    );
    let retained: (String, i64, i64, Option<i64>) = sqlx::query_as("SELECT state,estimated_input_tokens,max_output_tokens,retry_authorized_at FROM spaces_assistant_model_calls WHERE id='lost'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(
        (retained.0.as_str(), retained.1, retained.2),
        ("outcome_unknown", 100, 100)
    );
    assert!(retained.3.is_some());
}
