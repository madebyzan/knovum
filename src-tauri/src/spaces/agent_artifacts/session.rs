use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::{ai::contracts::ExternalAgentKind, spaces::db};

pub(super) async fn start(
    app: &AppHandle,
    session_id: &str,
    space_id: &str,
    kind: ExternalAgentKind,
    prompt: &str,
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let principal = crate::spaces::authorization::Principal {
        kind: crate::spaces::authorization::PrincipalKind::ExternalAgent,
        id: session_id.to_owned(),
    };
    let scope = crate::spaces::authorization::RunScope::Space {
        space_id: space_id.to_owned(),
    };
    crate::spaces::runs::create_run_in_transaction(
        &mut tx,
        &crate::spaces::runs::NewRun {
            id: session_id,
            scope: &scope,
            parent_run_id: None,
            principal: &principal,
            kind: crate::spaces::runs::RunKind::ExternalAgent,
            policy_snapshot: &serde_json::json!({"runtimeKind": kind}),
            input_snapshot: &serde_json::json!({"prompt": prompt}),
            budget: &serde_json::json!({"maxDurationSeconds": 7200}),
        },
        now,
    )
    .await?;
    sqlx::query("UPDATE spaces_runs SET state='running',state_version=1 WHERE id=$1")
        .bind(session_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    sqlx::query(
        "INSERT INTO spaces_agent_artifact_sessions(
          id,run_id,space_id,runtime_kind,state,error,created_at,updated_at,completed_at
        ) VALUES($1,$1,$2,$3,'running',NULL,$4,$4,NULL)",
    )
    .bind(session_id)
    .bind(space_id)
    .bind(kind.as_str())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn succeed(app: &AppHandle, session_id: &str) -> Result<(), String> {
    finish(db::pool(app).await?, session_id, "completed", None).await
}

pub(super) async fn fail(app: &AppHandle, session_id: &str, error: &str) -> Result<(), String> {
    let error = error.trim();
    if error.is_empty() {
        return Err("Agent artifact failure requires an error".into());
    }
    finish(db::pool(app).await?, session_id, "failed", Some(error)).await
}

pub(super) async fn cancel(app: &AppHandle, session_id: &str) -> Result<(), String> {
    finish(db::pool(app).await?, session_id, "cancelled", None).await
}

async fn finish(
    pool: &SqlitePool,
    session_id: &str,
    state: &str,
    error: Option<&str>,
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let session_write = sqlx::query(
        "UPDATE spaces_agent_artifact_sessions SET state=$1,error=$2,updated_at=$3,completed_at=$3
         WHERE id=$4 AND state='running'",
    )
    .bind(state)
    .bind(error)
    .bind(now)
    .bind(session_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(session_write, "Agent artifact session is already terminal")?;
    let run_state = if state == "completed" {
        "succeeded"
    } else {
        state
    };
    sqlx::query(
        "UPDATE spaces_runs SET state=$1,state_version=state_version+1,updated_at=$2,
         completed_at=$2 WHERE id=$3 AND completed_at IS NULL",
    )
    .bind(run_state)
    .bind(now)
    .bind(session_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn completed_at(
    app: &AppHandle,
    space_id: &str,
    session_id: &str,
) -> Result<Option<i64>, String> {
    sqlx::query_scalar(
        "SELECT completed_at FROM spaces_agent_artifact_sessions
         WHERE id=$1 AND space_id=$2 AND state='completed'",
    )
    .bind(session_id)
    .bind(space_id)
    .fetch_optional(db::pool(app).await?)
    .await
    .map_err(db::sql_error)
}

pub(super) async fn fail_interrupted(app: &AppHandle) -> Result<Vec<(String, String)>, String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id,space_id FROM spaces_agent_artifact_sessions WHERE state='running'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_agent_artifact_sessions SET state='failed',
         error='Agent artifact execution was interrupted',updated_at=$1,completed_at=$1
         WHERE state='running'",
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='failed',state_version=state_version+1,updated_at=$1,
         completed_at=$1 WHERE completed_at IS NULL AND id IN
         (SELECT run_id FROM spaces_agent_artifact_sessions WHERE state='failed')",
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(rows)
}
