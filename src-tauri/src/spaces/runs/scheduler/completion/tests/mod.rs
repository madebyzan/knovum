use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use super::{complete_step, settle_waiting_step, wait_step};
use crate::spaces::runs::{AttemptClaim, WaitReason};

struct Fixture {
    pool: SqlitePool,
    claim: AttemptClaim,
}

async fn executing_fixture() -> Fixture {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,
         execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at)
         VALUES('run','space','assistant','principal','action','running',7,$1,'{}',$2,1,1)",
    )
    .bind(r#"{"definition":{"failurePolicy":"stop_run"}}"#)
    .bind(r#"{"maxOutputBytes":1024}"#)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,
         resolved_input,created_at,updated_at) VALUES('step','run','node','node','action','running','{}',1,1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,
         request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at)
         VALUES('attempt','run','step',1,'executing',$1,'executor','step',7,999999,1)",
    )
    .bind("0".repeat(64))
    .execute(&pool)
    .await
    .unwrap();
    Fixture {
        pool,
        claim: AttemptClaim {
            attempt_id: "attempt".into(),
            run_id: "run".into(),
            step_instance_id: "step".into(),
            execution_epoch: 7,
            attempt_number: 1,
            home_space_id: "space".into(),
            node_kind: "action".into(),
            configuration: json!({}),
            resolved_input: json!({}),
            policy_snapshot: json!({"definition": {"failurePolicy": "stop_run"}}),
            budget: json!({"maxOutputBytes": 1024}),
        },
    }
}

async fn add_pending_dependent(pool: &SqlitePool) {
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,
         depends_on,state,resolved_input,created_at,updated_at)
         VALUES('next','run','next','next','action','[\"node\"]','pending','{}',1,1)",
    )
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn cancellation_after_claim_rejects_completion_without_reserving_output() {
    let fixture = executing_fixture().await;
    crate::spaces::runs::repository::request_control(&fixture.pool, "run", "cancel")
        .await
        .unwrap();

    let error = complete_step(
        &fixture.pool,
        &fixture.claim,
        &json!({"value": "late"}),
        None,
    )
    .await
    .unwrap_err();

    assert!(error.code == "attempt_stale");
    let row: (String, String, i64, Option<i64>) = sqlx::query_as(
        "SELECT a.state,s.state,r.output_bytes,r.cancel_requested_at
         FROM spaces_run_attempts a JOIN spaces_run_steps s ON s.id=a.step_instance_id
         JOIN spaces_runs r ON r.id=a.run_id WHERE a.id='attempt'",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(row.0, "executing");
    assert_eq!(row.1, "running");
    assert_eq!(row.2, 0);
    assert!(row.3.is_some());
}

#[tokio::test]
async fn stale_run_epoch_rejects_completion_without_reserving_output() {
    let fixture = executing_fixture().await;
    sqlx::query("UPDATE spaces_runs SET execution_epoch=execution_epoch+1 WHERE id='run'")
        .execute(&fixture.pool)
        .await
        .unwrap();

    assert!(
        complete_step(
            &fixture.pool,
            &fixture.claim,
            &json!({"value": "stale"}),
            None
        )
        .await
        .unwrap_err()
        .code
            == "attempt_stale"
    );
    let bytes: i64 = sqlx::query_scalar("SELECT output_bytes FROM spaces_runs WHERE id='run'")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(bytes, 0);
}

#[tokio::test]
async fn sibling_completion_remains_authoritative_after_run_enters_waiting() {
    let fixture = executing_fixture().await;
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,
         resolved_input,created_at,updated_at) VALUES('sibling','run','sibling','sibling','action','running','{}',1,1)",
    )
    .execute(&fixture.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,
         request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at)
         VALUES('sibling-attempt','run','sibling',1,'executing',$1,'sibling-executor','sibling',7,999999,1)",
    )
    .bind("1".repeat(64))
    .execute(&fixture.pool)
    .await
    .unwrap();
    let mut sibling = fixture.claim.clone();
    sibling.attempt_id = "sibling-attempt".into();
    sibling.step_instance_id = "sibling".into();

    wait_step(
        &fixture.pool,
        &fixture.claim,
        WaitReason::Executor,
        &json!({"childRunId": "child"}),
    )
    .await
    .unwrap();
    complete_step(&fixture.pool, &sibling, &json!({"done": true}), None)
        .await
        .unwrap();

    let row: (String, String, i64) = sqlx::query_as(
        "SELECT r.state,a.state,r.output_bytes FROM spaces_runs r
         JOIN spaces_run_attempts a ON a.run_id=r.id WHERE a.id='sibling-attempt'",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(row.0, "waiting");
    assert_eq!(row.1, "succeeded");
    assert!(row.2 > 0);
}

#[tokio::test]
async fn pause_allows_current_attempt_to_finish_then_controls_pause_the_run() {
    let fixture = executing_fixture().await;
    add_pending_dependent(&fixture.pool).await;
    crate::spaces::runs::repository::request_control(&fixture.pool, "run", "pause")
        .await
        .unwrap();
    let output = json!({"value": "accepted"});
    complete_step(&fixture.pool, &fixture.claim, &output, None)
        .await
        .unwrap();

    let before_control: (String, Option<i64>, i64) =
        sqlx::query_as("SELECT state,completed_at,output_bytes FROM spaces_runs WHERE id='run'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(before_control.0, "running");
    assert!(before_control.1.is_none());
    assert_eq!(
        before_control.2,
        i64::try_from(serde_json::to_vec(&output).unwrap().len()).unwrap()
    );

    super::super::controls::apply(&fixture.pool).await.unwrap();
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(state, "paused");
    let next: String = sqlx::query_scalar("SELECT state FROM spaces_run_steps WHERE id='next'")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(next, "ready");
}

#[tokio::test]
async fn pause_does_not_override_natural_completion_of_the_last_step() {
    let fixture = executing_fixture().await;
    crate::spaces::runs::repository::request_control(&fixture.pool, "run", "pause")
        .await
        .unwrap();

    complete_step(&fixture.pool, &fixture.claim, &json!({"done": true}), None)
        .await
        .unwrap();

    let row: (String, Option<i64>) =
        sqlx::query_as("SELECT state,completed_at FROM spaces_runs WHERE id='run'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(row.0, "succeeded");
    assert!(row.1.is_some());
}

#[tokio::test]
async fn waiting_outcome_pauses_after_drain_and_settles_after_resume() {
    let fixture = executing_fixture().await;
    crate::spaces::runs::repository::request_control(&fixture.pool, "run", "pause")
        .await
        .unwrap();
    let checkpoint = json!({"childRunId": "child"});
    wait_step(
        &fixture.pool,
        &fixture.claim,
        WaitReason::Executor,
        &checkpoint,
    )
    .await
    .unwrap();

    super::super::controls::apply(&fixture.pool).await.unwrap();
    let paused: (String, Option<String>) =
        sqlx::query_as("SELECT state,wait_reason FROM spaces_runs WHERE id='run'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(paused, ("paused".into(), None));

    crate::spaces::runs::repository::request_control(&fixture.pool, "run", "resume")
        .await
        .unwrap();
    let result = json!({"done": true});
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    let version = *changes.borrow_and_update();
    settle_waiting_step(&fixture.pool, "step", "run", Ok(&result))
        .await
        .unwrap();
    assert_ne!(*changes.borrow_and_update(), version);
    let completed: (String, Option<i64>) =
        sqlx::query_as("SELECT state,completed_at FROM spaces_runs WHERE id='run'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(completed.0, "succeeded");
    assert!(completed.1.is_some());
}

#[tokio::test]
async fn cancelled_waiting_run_rejects_late_settlement() {
    let fixture = executing_fixture().await;
    sqlx::query("UPDATE spaces_run_attempts SET state='succeeded' WHERE id='attempt'")
        .execute(&fixture.pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE spaces_run_steps SET state='waiting',wait_reason='executor' WHERE id='step'",
    )
    .execute(&fixture.pool)
    .await
    .unwrap();
    sqlx::query("UPDATE spaces_runs SET state='waiting',wait_reason='executor' WHERE id='run'")
        .execute(&fixture.pool)
        .await
        .unwrap();
    crate::spaces::runs::repository::request_control(&fixture.pool, "run", "cancel")
        .await
        .unwrap();

    let settlement = json!({"done": true});
    let error = settle_waiting_step(&fixture.pool, "step", "run", Ok(&settlement))
        .await
        .unwrap_err();
    assert!(error.starts_with("run-step-stale:"));
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_run_steps WHERE id='step'")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(state, "waiting");
}

#[tokio::test]
async fn cancellation_after_atomic_completion_is_rejected_as_already_complete() {
    let fixture = executing_fixture().await;
    complete_step(&fixture.pool, &fixture.claim, &json!({"done": true}), None)
        .await
        .unwrap();

    let error = crate::spaces::runs::repository::request_control(&fixture.pool, "run", "cancel")
        .await
        .unwrap_err();
    assert_eq!(error, "Run is unavailable or already complete");
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(state, "succeeded");
}

#[tokio::test]
async fn failure_policy_uses_codes_and_records_the_complete_envelope() {
    let fixture = executing_fixture().await;
    let error =
        crate::app_error::AppError::new("authentication_failed", "rate-limited: misleading prose");
    super::fail_step(&fixture.pool, &fixture.claim, &error)
        .await
        .unwrap();
    let code: String =
        sqlx::query_scalar("SELECT error_code FROM spaces_run_attempts WHERE id='attempt'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(code, "authentication_failed");
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_run_steps WHERE id='step'")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(state, "failed");
    let payload: String =
        sqlx::query_scalar("SELECT payload FROM spaces_run_events WHERE kind='step_failed'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&payload).unwrap()["error"],
        serde_json::to_value(&error).unwrap()
    );
    assert!(super::retryable(&crate::app_error::AppError::new(
        "rate_limited",
        "Unrelated prose"
    )));
    assert!(!super::retryable(&error));
}
