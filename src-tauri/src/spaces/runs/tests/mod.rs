use super::{load_run, RunKind};

#[tokio::test]
async fn database_run_kinds_decode_into_the_rust_contract() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool)
        .await
        .unwrap();
    for (index, kind) in [
        "action",
        "workflow",
        "builtin_agent",
        "external_agent",
        "automation",
    ]
    .into_iter()
    .enumerate()
    {
        let id = format!("run-{index}");
        sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at) VALUES($1,'space','space','user','user',$2,'succeeded','{}','{}','{}',1,1,1)")
            .bind(&id)
            .bind(kind)
            .execute(&pool)
            .await
            .unwrap();
        let run = load_run(&pool, &id).await.unwrap();
        if kind == "automation" {
            assert_eq!(run.kind, RunKind::Automation);
        }
    }
}

#[tokio::test]
async fn application_runs_cannot_create_generic_retries_or_grants() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO spaces_runs(
          id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,
          input_snapshot,budget,created_at,updated_at,completed_at
         ) VALUES(
          'candidate','application',NULL,'user','local-user','workflow','succeeded','{}',
          '{\"workflow\":\"speech_voice_candidate_v1\"}','{}',1,1,1
         )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let error = super::retry::create_retry(&pool, "candidate", &[])
        .await
        .unwrap_err();
    assert_eq!(
        error,
        "run-retry-unsupported: only user-started Space workflows can be retried"
    );
    let run_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_runs")
        .fetch_one(&pool)
        .await
        .unwrap();
    let grant_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_authorization_grants")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(run_count, 1);
    assert_eq!(grant_count, 0);
}
