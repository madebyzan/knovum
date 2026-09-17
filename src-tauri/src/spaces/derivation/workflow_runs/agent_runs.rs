use sqlx::Row;
use tauri::AppHandle;

use crate::{ai::contracts::AgentKind, spaces::db};

const WORKFLOW_ID: &str = "agent-artifact-creation";
const WORKFLOW_VERSION: i64 = 1;
const STEP_ID: &str = "create-artifacts";
const INTERRUPTED: &str = "Agent artifact creation was interrupted by application restart";

pub struct InterruptedAgentRun {
    pub session_id: String,
    pub space_id: String,
}

pub async fn start(
    app: &AppHandle,
    session_id: &str,
    space_id: &str,
    kind: AgentKind,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let space_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(space_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    if space_exists != 1 {
        return Err("Agent artifact creation Space does not exist".into());
    }
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_creation_workflow_runs(id,space_id,workflow_id,workflow_version,status,error,created_at,updated_at,completed_at) \
         VALUES($1,$2,$3,$4,'running','',$5,$5,NULL)",
    )
    .bind(session_id)
    .bind(space_id)
    .bind(WORKFLOW_ID)
    .bind(WORKFLOW_VERSION)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "INSERT INTO spaces_creation_workflow_steps(run_id,step_id,action,status,output_node_ids,error,created_at,updated_at,completed_at) \
         VALUES($1,$2,$3,'running','[]','',$4,$4,NULL)",
    )
    .bind(session_id)
    .bind(STEP_ID)
    .bind(kind.as_str())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn succeed(app: &AppHandle, session_id: &str) -> Result<(), String> {
    finish(app, session_id, "completed", "succeeded", "").await
}

pub async fn fail(app: &AppHandle, session_id: &str, error: &str) -> Result<(), String> {
    let error = error.trim();
    if error.is_empty() {
        return Err("Agent artifact creation failure requires an error".into());
    }
    finish(app, session_id, "failed", "failed", error).await
}

pub async fn cancel(app: &AppHandle, session_id: &str) -> Result<(), String> {
    finish(app, session_id, "cancelled", "cancelled", "").await
}

async fn finish(
    app: &AppHandle,
    session_id: &str,
    run_status: &str,
    step_status: &str,
    error: &str,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    let step = sqlx::query(
        "UPDATE spaces_creation_workflow_steps SET status=$1,error=$2,updated_at=$3,completed_at=$3 \
         WHERE run_id=$4 AND step_id=$5 AND status='running'",
    )
    .bind(step_status)
    .bind(error)
    .bind(now)
    .bind(session_id)
    .bind(STEP_ID)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(step, "Agent artifact creation step is already terminal")?;
    let run = sqlx::query(
        "UPDATE spaces_creation_workflow_runs SET status=$1,error=$2,updated_at=$3,completed_at=$3 \
         WHERE id=$4 AND workflow_id=$5 AND status='running'",
    )
    .bind(run_status)
    .bind(error)
    .bind(now)
    .bind(session_id)
    .bind(WORKFLOW_ID)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(run, "Agent artifact creation run is already terminal")?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn completed_at(
    app: &AppHandle,
    space_id: &str,
    session_id: &str,
) -> Result<Option<i64>, String> {
    let pool = db::pool(app).await?;
    sqlx::query_scalar(
        "SELECT completed_at FROM spaces_creation_workflow_runs \
         WHERE id=$1 AND space_id=$2 AND workflow_id=$3 AND status='completed'",
    )
    .bind(session_id)
    .bind(space_id)
    .bind(WORKFLOW_ID)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)
}

pub async fn reconcile_interrupted(app: &AppHandle) -> Result<Vec<InterruptedAgentRun>, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let rows = sqlx::query(
        "SELECT id,space_id FROM spaces_creation_workflow_runs \
         WHERE workflow_id=$1 AND status='running'",
    )
    .bind(WORKFLOW_ID)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let runs = rows
        .into_iter()
        .map(|row| {
            Ok(InterruptedAgentRun {
                session_id: row.try_get("id").map_err(db::sql_error)?,
                space_id: row.try_get("space_id").map_err(db::sql_error)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_creation_workflow_steps SET status='failed',error=$1,updated_at=$2,completed_at=$2 \
         WHERE status='running' AND run_id IN \
         (SELECT id FROM spaces_creation_workflow_runs WHERE workflow_id=$3 AND status='running')",
    )
    .bind(INTERRUPTED)
    .bind(now)
    .bind(WORKFLOW_ID)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_creation_workflow_runs SET status='failed',error=$1,updated_at=$2,completed_at=$2 \
         WHERE workflow_id=$3 AND status='running'",
    )
    .bind(INTERRUPTED)
    .bind(now)
    .bind(WORKFLOW_ID)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(runs)
}
