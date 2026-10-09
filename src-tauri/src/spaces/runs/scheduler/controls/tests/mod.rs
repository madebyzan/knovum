use sqlx::sqlite::SqlitePoolOptions;

use super::{apply, recover_expired_leases};

#[tokio::test]
async fn explicit_cancellation_wins_over_duration_expiry_in_the_same_tick() {
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
             execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at)
             VALUES('run','space','space','builtin_agent','principal','action','running',3,'{}','{}',
             '{\"maxDurationSeconds\":0}',1,1)",
    )
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
             VALUES('attempt','run','step',1,'executing',$1,'executor','step',3,9999999999999,1)",
    )
    .bind("0".repeat(64))
    .execute(&pool)
    .await
    .unwrap();

    crate::spaces::runs::repository::request_control(&pool, "run", "cancel")
        .await
        .unwrap();
    apply(&pool).await.unwrap();

    let states: (
        String,
        Option<String>,
        String,
        Option<String>,
        String,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT r.state,r.wait_reason,s.state,s.wait_reason,a.state,a.error_code
                 FROM spaces_runs r JOIN spaces_run_steps s ON s.run_id=r.id
                 JOIN spaces_run_attempts a ON a.run_id=r.id WHERE r.id='run'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(states.0, "cancelled");
    assert!(states.1.is_none());
    assert_eq!(states.2, "cancelled");
    assert!(states.3.is_none());
    assert_eq!(states.4, "cancelled");
    assert_eq!(states.5.as_deref(), Some("run_cancelled"));
}

#[tokio::test]
async fn expired_lease_recovery_transitions_each_run_once_even_when_pause_is_pending() {
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
             state_version,execution_epoch,policy_snapshot,input_snapshot,budget,pause_requested_at,
             created_at,updated_at) VALUES('run','space','space','builtin_agent','principal','action','running',
             0,3,'{}','{}','{}',2,1,1)",
    )
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
             VALUES('attempt','run','step',1,'executing',$1,'executor','step',3,0,1)",
    )
    .bind("0".repeat(64))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
            "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,
             resolved_input,created_at,updated_at) VALUES('sibling','run','sibling','sibling','action','running','{}',1,1)",
        )
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,
             request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at)
             VALUES('sibling-attempt','run','sibling',1,'executing',$1,'sibling-executor',
             'sibling',3,9999999999999,1)",
    )
    .bind("1".repeat(64))
    .execute(&pool)
    .await
    .unwrap();

    let mut changes = crate::spaces::runs::subscribe_to_changes();
    let version = *changes.borrow_and_update();
    recover_expired_leases(&pool).await.unwrap();
    assert_ne!(*changes.borrow_and_update(), version);
    apply(&pool).await.unwrap();
    let first: (String, i64, i64, String, String) = sqlx::query_as(
        "SELECT r.state,r.state_version,r.execution_epoch,s.state,s.wait_reason
             FROM spaces_runs r JOIN spaces_run_steps s ON s.run_id=r.id WHERE r.id='run'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        first,
        (
            "paused".into(),
            2,
            4,
            "blocked".into(),
            "external_reconciliation".into()
        )
    );
    let attempts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_run_attempts WHERE run_id='run' AND state='outcome_unknown'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let steps: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_run_steps WHERE run_id='run'
             AND state='blocked' AND wait_reason='external_reconciliation'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((attempts, steps), (2, 2));

    sqlx::query("UPDATE spaces_runs SET updated_at=9000000000001 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    recover_expired_leases(&pool).await.unwrap();
    apply(&pool).await.unwrap();
    let second: (String, i64, i64, i64, String, String) = sqlx::query_as(
        "SELECT r.state,r.state_version,r.execution_epoch,r.updated_at,s.state,s.wait_reason
             FROM spaces_runs r JOIN spaces_run_steps s ON s.run_id=r.id WHERE r.id='run'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        second,
        (
            "paused".into(),
            2,
            4,
            9_000_000_000_001,
            "blocked".into(),
            "external_reconciliation".into(),
        )
    );
}

#[tokio::test]
async fn cancellation_propagation_is_idempotent_through_cancelled_intermediates() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
    insert_run(&pool, "root", None, "cancelled", 4, Some(10), Some(10)).await;
    insert_run(
        &pool,
        "intermediate",
        Some("root"),
        "cancelled",
        5,
        Some(11),
        Some(11),
    )
    .await;
    insert_run(&pool, "late", Some("intermediate"), "queued", 6, None, None).await;

    apply(&pool).await.unwrap();
    let propagated: (String, i64, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT state,execution_epoch,cancel_requested_at,completed_at
             FROM spaces_runs WHERE id='late'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(propagated.0, "cancelled");
    assert_eq!(propagated.1, 7);
    assert!(propagated.2.is_some());
    assert!(propagated.3.is_some());

    sqlx::query(
        "UPDATE spaces_runs SET updated_at=CASE id
             WHEN 'root' THEN 9000000000001 WHEN 'intermediate' THEN 9000000000002
             ELSE 9000000000003 END WHERE id IN ('root','intermediate','late')",
    )
    .execute(&pool)
    .await
    .unwrap();
    apply(&pool).await.unwrap();

    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT id,execution_epoch,updated_at FROM spaces_runs
             WHERE id IN ('root','intermediate','late') ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("intermediate".into(), 5, 9_000_000_000_002),
            ("late".into(), 7, 9_000_000_000_003),
            ("root".into(), 4, 9_000_000_000_001),
        ]
    );
}

#[tokio::test]
async fn duration_expiry_fences_the_entire_unfinished_run_tree() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('parent','space','space','builtin_agent','principal','workflow','running',2,'{}','{}','{\"maxDurationSeconds\":0}',1,1)")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('child','space','space','parent','builtin_agent','principal','action','running',7,'{}','{}','{}',1,1)")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,resolved_input,created_at,updated_at) VALUES('child-step','child','node','node','action','running','{}',1,1)")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at) VALUES('child-attempt','child','child-step',1,'executing',$1,'executor','child-step',7,9999999999999,1)")
            .bind("0".repeat(64)).execute(&pool).await.unwrap();

    insert_run(
        &pool,
        "completed",
        Some("parent"),
        "succeeded",
        9,
        None,
        Some(3),
    )
    .await;
    insert_run(
        &pool,
        "grandchild",
        Some("completed"),
        "queued",
        10,
        None,
        None,
    )
    .await;
    apply(&pool).await.unwrap();

    let runs: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT id,state,execution_epoch FROM spaces_runs WHERE id IN ('parent','child') ORDER BY id",
        ).fetch_all(&pool).await.unwrap();
    assert_eq!(
        runs,
        vec![
            ("child".into(), "failed".into(), 8),
            ("parent".into(), "failed".into(), 3)
        ]
    );
    let preserved: (String, i64, i64) = sqlx::query_as(
        "SELECT state,execution_epoch,updated_at FROM spaces_runs WHERE id='completed'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(preserved, ("succeeded".into(), 9, 1));
    let grandchild: (String, i64) =
        sqlx::query_as("SELECT state,execution_epoch FROM spaces_runs WHERE id='grandchild'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(grandchild, ("failed".into(), 11));
    let execution: (String, String, Option<String>) = sqlx::query_as(
            "SELECT s.state,a.state,a.error_code FROM spaces_run_steps s JOIN spaces_run_attempts a ON a.step_instance_id=s.id WHERE s.id='child-step'",
        ).fetch_one(&pool).await.unwrap();
    assert_eq!(
        execution,
        (
            "blocked".into(),
            "cancelled".into(),
            Some("run_duration_exceeded".into())
        )
    );
}

async fn insert_run(
    pool: &sqlx::SqlitePool,
    id: &str,
    parent: Option<&str>,
    state: &str,
    epoch: i64,
    cancel_requested_at: Option<i64>,
    completed_at: Option<i64>,
) {
    sqlx::query(
            "INSERT INTO spaces_runs(id,scope_kind,home_space_id,parent_run_id,principal_kind,principal_id,
             kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,cancel_requested_at,
             created_at,updated_at,completed_at) VALUES($1,'space','space',$2,'builtin_agent','principal','action',
             $3,$4,'{}','{}','{}',$5,1,1,$6)",
        )
        .bind(id)
        .bind(parent)
        .bind(state)
        .bind(epoch)
        .bind(cancel_requested_at)
        .bind(completed_at)
        .execute(pool)
        .await
        .unwrap();
}
