use sqlx::sqlite::SqlitePoolOptions;

use super::claim_ready_step;

#[tokio::test]
async fn paused_ancestor_blocks_new_child_claims_until_resume() {
    let pool = fixture().await;
    insert_run(&pool, "parent", None, "workflow", "paused", "{}", Some(2)).await;
    insert_run(
        &pool,
        "child",
        Some("parent"),
        "action",
        "queued",
        "{}",
        None,
    )
    .await;
    insert_step(&pool, "candidate", "child", "ready").await;

    assert!(claim_ready_step(&pool, "executor").await.unwrap().is_none());
    crate::spaces::runs::repository::request_control(&pool, "parent", "resume")
        .await
        .unwrap();

    let claim = claim_ready_step(&pool, "executor").await.unwrap().unwrap();
    assert_eq!(claim.run_id, "child");
    assert_eq!(claim.step_instance_id, "candidate");
}

#[tokio::test]
async fn ancestor_parallelism_counts_running_steps_across_descendants() {
    let pool = fixture().await;
    insert_run(
        &pool,
        "parent",
        None,
        "workflow",
        "running",
        "{\"maxParallelism\":1}",
        None,
    )
    .await;
    insert_run(
        &pool,
        "active",
        Some("parent"),
        "action",
        "running",
        "{}",
        None,
    )
    .await;
    insert_run(
        &pool,
        "queued",
        Some("parent"),
        "action",
        "queued",
        "{}",
        None,
    )
    .await;
    insert_step(&pool, "active-step", "active", "running").await;
    insert_step(&pool, "candidate", "queued", "ready").await;

    assert!(claim_ready_step(&pool, "executor").await.unwrap().is_none());
}

#[tokio::test]
async fn exhausted_ancestor_attempt_budget_fences_its_unfinished_tree_and_announces() {
    let pool = fixture().await;
    insert_run(
        &pool,
        "parent",
        None,
        "workflow",
        "running",
        "{\"maxStepAttempts\":0,\"maxParallelism\":3}",
        None,
    )
    .await;
    insert_run(
        &pool,
        "candidate-run",
        Some("parent"),
        "action",
        "queued",
        "{}",
        None,
    )
    .await;
    insert_run(
        &pool,
        "active-run",
        Some("parent"),
        "action",
        "running",
        "{}",
        None,
    )
    .await;
    insert_step(&pool, "candidate", "candidate-run", "ready").await;
    insert_step(&pool, "active", "active-run", "running").await;
    sqlx::query(
        "INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,
         request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at)
         VALUES('active-attempt','active-run','active',1,'executing',$1,'active-executor',
         'active',0,9999999999999,1)",
    )
    .bind("0".repeat(64))
    .execute(&pool)
    .await
    .unwrap();
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    let version = *changes.borrow_and_update();

    assert!(claim_ready_step(&pool, "executor").await.unwrap().is_none());
    assert_ne!(*changes.borrow_and_update(), version);

    let runs: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT id,state,execution_epoch FROM spaces_runs
         WHERE id IN ('parent','candidate-run','active-run') ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        runs,
        vec![
            ("active-run".into(), "failed".into(), 1),
            ("candidate-run".into(), "failed".into(), 1),
            ("parent".into(), "failed".into(), 1),
        ]
    );
    let states: (String, String, Option<String>) = sqlx::query_as(
        "SELECT s.state,a.state,a.error_code FROM spaces_run_steps s
         JOIN spaces_run_attempts a ON a.step_instance_id=s.id WHERE s.id='active'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        states,
        (
            "blocked".into(),
            "cancelled".into(),
            Some("run-attempt-budget-exceeded".into())
        )
    );
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
    pool
}

async fn insert_run(
    pool: &sqlx::SqlitePool,
    id: &str,
    parent: Option<&str>,
    kind: &str,
    state: &str,
    budget: &str,
    pause_requested_at: Option<i64>,
) {
    sqlx::query(
        "INSERT INTO spaces_runs(id,scope_kind,home_space_id,parent_run_id,principal_kind,principal_id,kind,
         state,policy_snapshot,input_snapshot,budget,pause_requested_at,created_at,updated_at)
         VALUES($1,'space','space',$2,'builtin_agent','principal',$3,$4,'{}','{}',$5,$6,1,1)",
    )
    .bind(id)
    .bind(parent)
    .bind(kind)
    .bind(state)
    .bind(budget)
    .bind(pause_requested_at)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_step(pool: &sqlx::SqlitePool, id: &str, run_id: &str, state: &str) {
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,
         resolved_input,created_at,updated_at) VALUES($1,$2,$1,$1,'action',$3,'{}',1,1)",
    )
    .bind(id)
    .bind(run_id)
    .bind(state)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn blocked_child_does_not_starve_an_unrelated_run_and_successful_claim_announces() {
    let pool = fixture().await;
    insert_run(&pool, "parent", None, "workflow", "paused", "{}", Some(2)).await;
    insert_run(
        &pool,
        "blocked",
        Some("parent"),
        "action",
        "queued",
        "{}",
        None,
    )
    .await;
    insert_step(&pool, "blocked-step", "blocked", "ready").await;
    insert_run(&pool, "independent", None, "workflow", "queued", "{}", None).await;
    insert_step(&pool, "independent-step", "independent", "ready").await;
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    let version = *changes.borrow_and_update();
    let claim = claim_ready_step(&pool, "executor").await.unwrap().unwrap();
    assert_eq!(claim.run_id, "independent");
    assert_ne!(*changes.borrow_and_update(), version);
    let attempts: i64 =
        sqlx::query_scalar("SELECT step_attempts FROM spaces_runs WHERE id='blocked'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(attempts, 0);
}

#[tokio::test]
async fn previously_accepted_child_can_drain_after_parent_failure() {
    let pool = fixture().await;
    insert_run(&pool, "parent", None, "builtin_agent", "failed", "{}", None).await;
    sqlx::query("UPDATE spaces_runs SET completed_at=2 WHERE id='parent'")
        .execute(&pool)
        .await
        .unwrap();
    insert_run(
        &pool,
        "child",
        Some("parent"),
        "action",
        "queued",
        "{}",
        None,
    )
    .await;
    insert_step(&pool, "candidate", "child", "ready").await;
    assert_eq!(
        claim_ready_step(&pool, "executor")
            .await
            .unwrap()
            .unwrap()
            .run_id,
        "child"
    );
}
