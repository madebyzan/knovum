//! Each migration upgrades the schema the previous release shipped. A test here starts from that
//! schema, carries representative rows through the migration, and checks that the upgraded library
//! is the one a fresh install creates, with the rows it had.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// A library exactly as `version` left it.
async fn shipped(version: i64) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::super::MIGRATOR
        .run_to(version, &pool)
        .await
        .expect("shipped schema");
    pool
}

async fn schema(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
        .fetch_all(pool)
        .await
        .expect("schema")
}

/// 0.0.1 recorded the sessions of the removed Create with Agent dialog. Upgrading settles the Run
/// a crash left open — nothing can finish it any more — and keeps every Run, while the session
/// table leaves with the feature.
#[tokio::test]
async fn upgrading_0_0_1_settles_and_retires_agent_artifact_sessions() {
    let pool = shipped(1).await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    for (run, state, completed_at) in [
        ("interrupted", "running", None),
        ("finished", "succeeded", Some(5_i64)),
        ("unrelated", "running", None),
    ] {
        sqlx::query(
            "INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind, \
             state,state_version,policy_snapshot,input_snapshot,budget,created_at,updated_at, \
             completed_at) \
             VALUES($1,'space','s','external_agent',$1,'external_agent',$2,1,'{}','{}','{}',1,1,$3)",
        )
        .bind(run)
        .bind(state)
        .bind(completed_at)
        .execute(&pool)
        .await
        .expect("run");
    }
    for (session, state) in [("interrupted", "running"), ("finished", "completed")] {
        sqlx::query(
            "INSERT INTO spaces_agent_artifact_sessions(id,run_id,space_id,runtime_kind,state, \
             created_at,updated_at) VALUES($1,$1,'s','codex',$2,1,1)",
        )
        .bind(session)
        .bind(state)
        .execute(&pool)
        .await
        .expect("session");
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let runs: Vec<(String, String, i64, Option<i64>)> =
        sqlx::query_as("SELECT id,state,state_version,completed_at FROM spaces_runs ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("runs");
    let settled: Vec<_> = runs
        .iter()
        .map(|(id, state, version, completed_at)| {
            (
                id.as_str(),
                state.as_str(),
                *version,
                completed_at.is_some_and(|at| at > 1),
            )
        })
        .collect();
    assert_eq!(
        settled,
        [
            ("finished", "succeeded", 1, true),
            ("interrupted", "failed", 2, true),
            ("unrelated", "running", 1, false),
        ]
    );
}
