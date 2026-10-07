use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use super::{next_deadline, wait};

async fn fixture() -> SqlitePool {
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

async fn run(pool: &SqlitePool, id: &str, budget: &str, completed: Option<i64>) {
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at)
        VALUES($1,'space','user','user','workflow','running','{}','{}',$2,1000,1000,$3)")
        .bind(id).bind(budget).bind(completed).execute(pool).await.unwrap();
}

async fn step(pool: &SqlitePool, id: &str, run: &str, state: &str, available_at: i64) {
    sqlx::query("INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,resolved_input,available_at,created_at,updated_at)
        VALUES($1,$2,$1,$1,'action',$3,'{}',$4,1,1)")
        .bind(id).bind(run).bind(state).bind(available_at).execute(pool).await.unwrap();
}

#[tokio::test]
async fn idle_history_has_no_deadline_and_duration_queries_use_unfinished_indexes() {
    let pool = fixture().await;
    run(&pool, "finished", "{\"maxDurationSeconds\":0}", Some(2000)).await;
    assert_eq!(next_deadline(&pool, 5000).await.unwrap(), None);
    let plans: Vec<(i64, i64, i64, String)> = sqlx::query_as(
        "EXPLAIN QUERY PLAN SELECT id FROM spaces_runs
        WHERE completed_at IS NULL AND cancel_requested_at IS NULL
        AND created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER)*1000 <= 5000",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(plans
        .iter()
        .any(|row| row.3.contains("spaces_runs_unfinished_deadline")));
    assert!(plans.iter().all(|row| !row.3.contains("SCAN spaces_runs")));
    let plans: Vec<(i64, i64, i64, String)> = sqlx::query_as(
        "EXPLAIN QUERY PLAN SELECT id FROM spaces_runs
        WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(plans
        .iter()
        .any(|row| row.3.contains("spaces_runs_unfinished_controls")));
}

#[tokio::test]
async fn future_retries_wake_at_availability_without_spinning_for_ready_blocked_work() {
    let pool = fixture().await;
    run(&pool, "run", "{\"maxDurationSeconds\":60}", None).await;
    step(&pool, "blocked", "run", "ready", 0).await;
    step(&pool, "retry", "run", "ready", 10000).await;
    assert_eq!(next_deadline(&pool, 5000).await.unwrap(), Some(10000));
    assert_eq!(next_deadline(&pool, 10000).await.unwrap(), Some(61000));
    sqlx::query("UPDATE spaces_runs SET pause_requested_at=1 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(next_deadline(&pool, 5000).await.unwrap(), Some(61000));
}

#[tokio::test]
async fn lease_deadline_matches_strict_expiry_and_ignores_obsolete_epochs() {
    let pool = fixture().await;
    run(&pool, "run", "{\"maxDurationSeconds\":60}", None).await;
    step(&pool, "step", "run", "running", 0).await;
    sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at)
        VALUES('attempt','run','step',1,'executing',$1,'executor','step',0,12000,1000)")
        .bind("0".repeat(64)).execute(&pool).await.unwrap();
    assert_eq!(next_deadline(&pool, 5000).await.unwrap(), Some(12001));
    sqlx::query("UPDATE spaces_runs SET execution_epoch=1 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(next_deadline(&pool, 5000).await.unwrap(), Some(61000));
}

#[tokio::test]
async fn signals_wake_an_idle_scheduler_and_overdue_deadlines_do_not_wait_for_polling() {
    let (sender, mut changes) = tokio::sync::watch::channel(0);
    sender.send_modify(|value| *value += 1);
    tokio::time::timeout(std::time::Duration::from_secs(1), wait(&mut changes, None))
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(
        std::time::Duration::from_secs(1),
        wait(&mut changes, Some(0)),
    )
    .await
    .unwrap()
    .unwrap();
}

#[tokio::test]
async fn idle_maintenance_does_not_wait_for_an_unrelated_sqlite_writer() {
    let directory = std::env::temp_dir().join(format!("knovum-scheduler-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(directory.join("runs.sqlite"))
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    let writer = crate::spaces::db::begin_write(&pool).await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        assert!(!super::super::controls::recover_expired_leases(&pool).await?);
        assert!(!super::super::controls::apply(&pool).await?);
        assert!(super::super::claim::claim_ready_step(&pool, "executor")
            .await?
            .is_none());
        Ok::<(), String>(())
    })
    .await
    .expect("idle scheduling must not acquire a write transaction")
    .unwrap();
    writer.rollback().await.unwrap();
    pool.close().await;
    std::fs::remove_dir_all(directory).unwrap();
}
