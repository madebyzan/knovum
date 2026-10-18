use crate::foundation::app_error::AppError;
mod active_attempts;
mod attempt;
mod budget;
mod children;
mod contracts;
mod desktop_tasks;
pub(crate) use desktop_tasks::snapshot as desktop_tasks;
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
pub(crate) use budget::{record_tokens_in_transaction, record_usage};
pub(crate) use children::{child_settlement, drain_children, wait_for_children};
pub(crate) use contracts::{RunKind, RunObservation, RunState, StepState, WaitReason};
pub(crate) use events::append_event;
pub use events::RunEventKind;
pub(crate) use observation::{RunEvent, StepObservation};
pub(crate) use repository::{
    create_run_in_transaction, load_run, request_cancellation_in_transaction, transition_run,
    NewRun,
};
pub(crate) use runtime::{start as start_scheduler, SchedulerHooks};
pub(crate) use signals::{announce as announce_change, subscribe as subscribe_to_changes};

pub(crate) async fn request_cancellation(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    repository::request_control(pool, run_id, "cancel").await
}

pub(crate) async fn cancel_active_attempts(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    active_attempts::cancel_tree(pool, run_id).await
}

pub(crate) async fn request_pause(pool: &sqlx::SqlitePool, run_id: &str) -> Result<(), String> {
    repository::request_control(pool, run_id, "pause").await
}

pub(crate) async fn request_resume(pool: &sqlx::SqlitePool, run_id: &str) -> Result<(), String> {
    repository::request_control(pool, run_id, "resume").await
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
