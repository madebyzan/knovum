use super::database;
use crate::spaces::speech::{cancellation, run_start};

#[tokio::test]
async fn preparation_cancellation_transfers_to_the_committed_speech_run() {
    let pool = database().await;
    let job_id = uuid::Uuid::new_v4().to_string();
    let lease = cancellation::acquire(&job_id).unwrap();
    assert!(cancellation::cancel(&job_id).unwrap());
    assert!(lease.token().is_cancelled());
    assert!(!cancellation::cancel_project_run(&pool, &job_id)
        .await
        .unwrap());
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES(?,'space','space','user','user','workflow','queued','{}',?,'{}',1,1)")
        .bind(&job_id).bind(serde_json::json!({"workflow": run_start::WORKFLOW}).to_string())
        .execute(&pool).await.unwrap();
    assert!(cancellation::cancel_project_run(&pool, &job_id)
        .await
        .unwrap());
    let requested: Option<i64> =
        sqlx::query_scalar("SELECT cancel_requested_at FROM spaces_runs WHERE id=?")
            .bind(&job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(requested.is_some());
    drop(lease);
    assert!(!cancellation::cancel(&job_id).unwrap());
    assert!(cancellation::cancel_project_run(&pool, &job_id)
        .await
        .unwrap());
}

#[tokio::test]
async fn preview_cancellation_does_not_control_an_unrelated_workflow() {
    let pool = database().await;
    let job_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES(?,'space','space','user','user','workflow','queued','{}','{}','{}',1,1)")
        .bind(&job_id).execute(&pool).await.unwrap();
    assert!(!cancellation::cancel_project_run(&pool, &job_id)
        .await
        .unwrap());
    let requested: Option<i64> =
        sqlx::query_scalar("SELECT cancel_requested_at FROM spaces_runs WHERE id=?")
            .bind(&job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(requested.is_none());
}
