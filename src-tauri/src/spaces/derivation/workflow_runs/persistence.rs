use std::collections::HashMap;

use sqlx::Row;
use tauri::{AppHandle, Runtime};

use super::contracts::{SaveWorkflowRunRequest, WorkflowStepStatus};
use super::validation;
use super::WorkflowRunRecord;
use crate::spaces::db;

pub async fn save<R: Runtime>(
    app: &AppHandle<R>,
    request: SaveWorkflowRunRequest,
) -> Result<WorkflowRunRecord, String> {
    let state = validation::validate(&request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    require_space(&mut tx, &request.space_id).await?;
    validate_outputs(&mut tx, &request).await?;
    validate_existing(&mut tx, &request, state.status, state.error).await?;
    let now = crate::clock::unix_time_millis()?;
    let completed_at = (state.status != "running").then_some(now);
    sqlx::query(
        "INSERT INTO spaces_creation_workflow_runs(id,space_id,workflow_id,workflow_version,status,error,created_at,updated_at,completed_at) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$7,$8) ON CONFLICT(id) DO UPDATE SET \
         status=excluded.status,error=excluded.error,updated_at=excluded.updated_at,completed_at=excluded.completed_at",
    )
    .bind(&request.id).bind(&request.space_id).bind(&request.workflow_id)
    .bind(request.workflow_version).bind(state.status).bind(state.error).bind(now).bind(completed_at)
    .execute(&mut *tx).await.map_err(db::sql_error)?;
    for step in &request.steps {
        let output_node_ids =
            serde_json::to_string(&step.output_node_ids).map_err(db::json_error)?;
        let completed_at = matches!(
            step.status,
            WorkflowStepStatus::Succeeded | WorkflowStepStatus::Failed
        )
        .then_some(now);
        sqlx::query(
            "INSERT INTO spaces_creation_workflow_steps(run_id,step_id,action,status,output_node_ids,error,created_at,updated_at,completed_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$7,$8) ON CONFLICT(run_id,step_id) DO UPDATE SET \
             status=excluded.status,output_node_ids=excluded.output_node_ids,error=excluded.error,updated_at=excluded.updated_at,completed_at=excluded.completed_at",
        )
        .bind(&request.id).bind(&step.id).bind(&step.action).bind(step.status.as_str())
        .bind(output_node_ids).bind(step.error.trim()).bind(now).bind(completed_at)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    super::queries::get(app, &request.id).await
}

async fn require_space(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if exists == 1 {
        Ok(())
    } else {
        Err("Creation Workflow Space does not exist".into())
    }
}

async fn validate_outputs(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &SaveWorkflowRunRequest,
) -> Result<(), String> {
    for node_id in request.steps.iter().flat_map(|step| &step.output_node_ids) {
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE id=$1 AND space_id=$2")
                .bind(node_id)
                .bind(&request.space_id)
                .fetch_one(&mut **tx)
                .await
                .map_err(db::sql_error)?;
        if exists != 1 {
            return Err("Creation Workflow output does not belong to its Space".into());
        }
    }
    Ok(())
}

async fn validate_existing(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &SaveWorkflowRunRequest,
    status: &str,
    error: &str,
) -> Result<(), String> {
    let row = sqlx::query(
        "SELECT space_id,workflow_id,workflow_version,status,error FROM spaces_creation_workflow_runs WHERE id=$1",
    )
    .bind(&request.id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(()) };
    let identity = row
        .try_get::<String, _>("space_id")
        .map_err(db::sql_error)?
        == request.space_id
        && row
            .try_get::<String, _>("workflow_id")
            .map_err(db::sql_error)?
            == request.workflow_id
        && row
            .try_get::<i64, _>("workflow_version")
            .map_err(db::sql_error)?
            == request.workflow_version;
    let prior_status: String = row.try_get("status").map_err(db::sql_error)?;
    let terminal = prior_status != "running";
    let same_terminal = prior_status == status
        && row.try_get::<String, _>("error").map_err(db::sql_error)? == error;
    if !identity || (terminal && !same_terminal) {
        return Err("Creation Workflow run identity or terminal state cannot change".into());
    }
    let rows = sqlx::query(
        "SELECT step_id,action,status FROM spaces_creation_workflow_steps WHERE run_id=$1",
    )
    .bind(&request.id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let existing = rows
        .into_iter()
        .map(|row| {
            Ok((
                row.try_get::<String, _>("step_id").map_err(db::sql_error)?,
                (
                    row.try_get::<String, _>("action").map_err(db::sql_error)?,
                    row.try_get::<String, _>("status").map_err(db::sql_error)?,
                ),
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    if existing.len() != request.steps.len()
        || request
            .steps
            .iter()
            .any(|step| match existing.get(&step.id) {
                None => true,
                Some((action, status)) => {
                    action != &step.action || !valid_step_transition(status, step.status.as_str())
                }
            })
    {
        return Err("Creation Workflow run step identity or terminal state cannot change".into());
    }
    Ok(())
}

fn valid_step_transition(previous: &str, next: &str) -> bool {
    matches!(
        (previous, next),
        ("pending", "pending" | "running")
            | ("running", "running" | "succeeded" | "failed")
            | ("succeeded", "succeeded")
            | ("failed", "failed")
    )
}

pub async fn reconcile_interrupted(app: &AppHandle) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    let error = "Application closed before the Creation Workflow finished";
    sqlx::query(
        "UPDATE spaces_creation_workflow_steps SET status='failed',error=$1,updated_at=$2,completed_at=$2 \
         WHERE status='running' OR (status='pending' AND run_id IN \
         (SELECT id FROM spaces_creation_workflow_runs WHERE status='running'))",
    )
        .bind(error).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_creation_workflow_runs SET status='failed',error=$1,updated_at=$2,completed_at=$2 WHERE status='running'")
        .bind(error).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}
