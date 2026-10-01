use crate::app_error::AppError;
mod attempt;
mod budget;
mod children;
mod contract;
mod events;
mod intervention;
pub(crate) mod observation;
pub(crate) mod observe;
mod repository;
mod retry;
mod run_rows;
mod runtime;
mod scheduler;
mod signals;

pub(crate) use attempt::{AttemptClaim, StepOutcome};
pub(crate) use budget::record_usage;
pub(crate) use children::{child_settlement, drain_children, wait_for_children};
pub(crate) use contract::{RunKind, RunObservation, RunState, StepState, WaitReason};
pub(crate) use events::append_event;
pub use events::RunEventKind;
pub(crate) use observation::{RunEvent, StepObservation};
pub(crate) use repository::{create_run_in_transaction, load_run, transition_run, NewRun};
pub(crate) use runtime::start as start_scheduler;
pub(crate) use signals::{announce as announce_change, subscribe as subscribe_to_changes};

pub(crate) async fn request_cancellation(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    repository::request_control(pool, run_id, "cancel").await
}

#[tauri::command]
pub async fn list_runs(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<RunObservation>, AppError> {
    Ok((repository::list_runs(crate::spaces::db::pool(&app).await?, &space_id).await)?)
}

#[tauri::command]
pub async fn get_run(app: tauri::AppHandle, run_id: String) -> Result<RunObservation, AppError> {
    Ok((load_run(crate::spaces::db::pool(&app).await?, &run_id).await)?)
}

#[tauri::command]
pub async fn cancel_run(app: tauri::AppHandle, run_id: String) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let run = load_run(pool, &run_id).await?;
    request_cancellation(pool, &run_id).await?;
    let action_runs: Vec<String> = sqlx::query_scalar(
        "WITH RECURSIVE run_tree(id) AS (
         SELECT id FROM spaces_runs WHERE id=$1
         UNION ALL SELECT child.id FROM spaces_runs child JOIN run_tree parent ON child.parent_run_id=parent.id)
         SELECT id FROM spaces_runs WHERE id IN (SELECT id FROM run_tree) AND kind='action' AND completed_at IS NULL",
    ).bind(&run_id).fetch_all(pool).await.map_err(crate::spaces::db::sql_error)?;
    for action_run in action_runs {
        if let Some(action_id) = repository::durable_action_id(pool, &action_run).await? {
            crate::spaces::derivation::action_execution::cancel(&action_id, &action_run).await?;
        }
    }
    if run.kind == RunKind::ExternalAgent
        && !crate::spaces::external_conversations::cancel_by_run(&app, &run_id).await?
    {
        crate::ai::executions::cancel(&run_id)?;
    }
    if run.kind == RunKind::Assistant {
        crate::spaces::assistant::cancel_task(&run_id)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pause_run(app: tauri::AppHandle, run_id: String) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    if load_run(pool, &run_id).await?.kind == RunKind::ExternalAgent {
        return Err(
            "run-control-unsupported: External Agent sessions support cancellation, not pause"
                .into(),
        );
    }
    Ok((repository::request_control(pool, &run_id, "pause").await)?)
}

#[tauri::command]
pub async fn resume_run(app: tauri::AppHandle, run_id: String) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let run = load_run(pool, &run_id).await?;
    if run.kind == RunKind::Assistant {
        crate::spaces::assistant::require_paused_checkpoint(pool, &run_id).await?;
    }
    repository::request_control(pool, &run_id, "resume").await?;
    if run.kind == RunKind::Assistant {
        crate::spaces::assistant::resume_task(app, &run_id).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn retry_run(
    app: tauri::AppHandle,
    run_id: String,
    step_ids: Vec<String>,
) -> Result<RunObservation, AppError> {
    Ok((retry::create_retry(crate::spaces::db::pool(&app).await?, &run_id, &step_ids).await)?)
}

#[tauri::command]
pub async fn list_run_steps(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Vec<observation::StepObservation>, AppError> {
    Ok((observation::list_steps(crate::spaces::db::pool(&app).await?, &run_id).await)?)
}

#[tauri::command]
pub async fn list_run_events(
    app: tauri::AppHandle,
    run_id: String,
    after_sequence: i64,
) -> Result<Vec<observation::RunEvent>, AppError> {
    Ok((observation::list_events(
        crate::spaces::db::pool(&app).await?,
        &run_id,
        after_sequence,
    )
    .await)?)
}

#[tauri::command]
pub async fn get_run_diagnostics(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<serde_json::Value, AppError> {
    Ok((observation::diagnostic_bundle(crate::spaces::db::pool(&app).await?, &run_id).await)?)
}

#[tauri::command]
pub async fn decide_run_step_review(
    app: tauri::AppHandle,
    step_id: String,
    expected_version: i64,
    reviewed_revision: String,
    approved: bool,
) -> Result<(), AppError> {
    Ok((intervention::decide_review(
        crate::spaces::db::pool(&app).await?,
        &step_id,
        expected_version,
        &reviewed_revision,
        approved,
    )
    .await)?)
}

#[tauri::command]
pub async fn reconcile_run_step(
    app: tauri::AppHandle,
    step_id: String,
    expected_version: i64,
    decision: intervention::ReconciliationDecision,
) -> Result<(), AppError> {
    Ok((intervention::reconcile(
        crate::spaces::db::pool(&app).await?,
        &step_id,
        expected_version,
        decision,
    )
    .await)?)
}
#[cfg(test)]
mod tests;
