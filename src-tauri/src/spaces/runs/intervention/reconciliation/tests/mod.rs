use sqlx::sqlite::SqlitePoolOptions;

use super::{reconcile, ReconciliationDecision};

#[tokio::test]
async fn cancelled_run_rejects_stale_reconciliation_without_changing_step_or_epoch() {
    let pool = fixture().await;
    crate::spaces::runs::repository::request_control(&pool, "run", "cancel")
        .await
        .unwrap();

    let error = reconcile(&pool, "step", 3, ReconciliationDecision::Fail)
        .await
        .unwrap_err();
    assert_eq!(error, "run-step-stale: reconciliation checkpoint changed");
    let row: (String, i64, Option<i64>, Option<i64>, String, i64) = sqlx::query_as(
        "SELECT r.state,r.execution_epoch,r.cancel_requested_at,r.completed_at,
             s.state,s.state_version FROM spaces_runs r JOIN spaces_run_steps s ON s.run_id=r.id
             WHERE r.id='run'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "waiting");
    assert_eq!(row.1, 5);
    assert!(row.2.is_some());
    assert!(row.3.is_none());
    assert_eq!(row.4, "blocked");
    assert_eq!(row.5, 3);
}

#[tokio::test]
async fn successful_reconciliation_announces_its_committed_transition() {
    let pool = fixture().await;
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    let version = *changes.borrow_and_update();

    reconcile(&pool, "step", 3, ReconciliationDecision::Retry)
        .await
        .unwrap();

    assert_ne!(*changes.borrow_and_update(), version);
    let row: (String, String) = sqlx::query_as(
        "SELECT r.state,s.state FROM spaces_runs r JOIN spaces_run_steps s ON s.run_id=r.id
             WHERE r.id='run'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row, ("queued".into(), "ready".into()));
}

async fn fixture() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
    sqlx::query(
            "INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,
             execution_epoch,wait_reason,policy_snapshot,input_snapshot,budget,created_at,updated_at)
             VALUES('run','space','space','builtin_agent','principal','action','waiting',4,
             'external_reconciliation','{}','{}','{}',1,1)",
        )
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,
             state_version,resolved_input,wait_reason,created_at,updated_at)
             VALUES('step','run','node','node','action','blocked',3,'{}',
             'external_reconciliation',1,1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}
