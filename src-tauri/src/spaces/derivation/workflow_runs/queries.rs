use sqlx::Row;
use tauri::{AppHandle, Runtime};

use super::contracts::{WorkflowRunRecord, WorkflowStepRecord};
use crate::spaces::db;

pub async fn get<R: Runtime>(app: &AppHandle<R>, id: &str) -> Result<WorkflowRunRecord, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query(
        "SELECT id,space_id,workflow_id,workflow_version,status,error,created_at,updated_at,completed_at \
         FROM spaces_creation_workflow_runs WHERE id=$1",
    )
    .bind(id).fetch_optional(pool).await.map_err(db::sql_error)?
    .ok_or_else(|| "Creation Workflow run does not exist".to_owned())?;
    record(pool, row).await
}

pub async fn list<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
) -> Result<Vec<WorkflowRunRecord>, String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query(
        "SELECT id,space_id,workflow_id,workflow_version,status,error,created_at,updated_at,completed_at \
         FROM spaces_creation_workflow_runs WHERE space_id=$1 ORDER BY created_at DESC,id DESC",
    )
    .bind(space_id).fetch_all(pool).await.map_err(db::sql_error)?;
    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        records.push(record(pool, row).await?);
    }
    Ok(records)
}

async fn record(
    pool: &sqlx::SqlitePool,
    row: sqlx::sqlite::SqliteRow,
) -> Result<WorkflowRunRecord, String> {
    let id: String = row.try_get("id").map_err(db::sql_error)?;
    let steps = sqlx::query(
        "SELECT step_id,action,status,output_node_ids,error,created_at,updated_at,completed_at \
         FROM spaces_creation_workflow_steps WHERE run_id=$1 ORDER BY created_at,step_id",
    )
    .bind(&id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?
    .into_iter()
    .map(step_record)
    .collect::<Result<Vec<_>, String>>()?;
    Ok(WorkflowRunRecord {
        id,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        workflow_id: row.try_get("workflow_id").map_err(db::sql_error)?,
        workflow_version: row.try_get("workflow_version").map_err(db::sql_error)?,
        status: row.try_get("status").map_err(db::sql_error)?,
        error: row.try_get("error").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
        completed_at: row.try_get("completed_at").map_err(db::sql_error)?,
        steps,
    })
}

fn step_record(row: sqlx::sqlite::SqliteRow) -> Result<WorkflowStepRecord, String> {
    let output: String = row.try_get("output_node_ids").map_err(db::sql_error)?;
    Ok(WorkflowStepRecord {
        id: row.try_get("step_id").map_err(db::sql_error)?,
        action: row.try_get("action").map_err(db::sql_error)?,
        status: row.try_get("status").map_err(db::sql_error)?,
        output_node_ids: serde_json::from_str(&output).map_err(db::json_error)?,
        error: row.try_get("error").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
        completed_at: row.try_get("completed_at").map_err(db::sql_error)?,
    })
}
