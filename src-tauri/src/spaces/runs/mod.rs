mod attempt;
mod budget;
mod contract;
mod events;
mod intervention;
pub(crate) mod observation;
mod repository;
mod retry;
mod run_rows;
mod runtime;
mod scheduler;
mod signals;

pub(crate) use attempt::{AttemptClaim, StepOutcome};
pub(crate) use budget::{
    record_usage, reserve_expanded_steps, reserve_expanded_steps_in_transaction,
    reserve_output_bytes,
};
pub(crate) use contract::{RunKind, RunState, RunSummary, StepState, WaitReason};
pub(crate) use events::append_event;
pub use events::RunEventKind;
pub(crate) use observation::BatchItemSummary;
pub(crate) use observation::{RunEvent, StepSummary};
pub(crate) use repository::{create_run_in_transaction, load_run, transition_run, NewRun};
pub(crate) use signals::{announce as announce_change, subscribe as subscribe_to_changes};

pub(crate) fn start_scheduler(app: tauri::AppHandle) {
    runtime::start(app);
}

#[tauri::command]
pub async fn list_runs(app: tauri::AppHandle, space_id: String) -> Result<Vec<RunSummary>, String> {
    repository::list_runs(crate::spaces::db::pool(&app).await?, &space_id).await
}

#[tauri::command]
pub async fn get_run(app: tauri::AppHandle, run_id: String) -> Result<RunSummary, String> {
    load_run(crate::spaces::db::pool(&app).await?, &run_id).await
}

#[tauri::command]
pub async fn cancel_run(app: tauri::AppHandle, run_id: String) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    let run = load_run(pool, &run_id).await?;
    repository::request_control(pool, &run_id, "cancel").await?;
    if let Some(action_id) = repository::durable_action_id(pool, &run_id).await? {
        crate::spaces::derivation::action_execution::cancel(&action_id, &run_id).await?;
    }
    for execution_id in repository::active_delegate_ids(pool, &run_id).await? {
        crate::ai::executions::cancel(&execution_id)?;
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
pub async fn pause_run(app: tauri::AppHandle, run_id: String) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    if load_run(pool, &run_id).await?.kind == RunKind::ExternalAgent {
        return Err(
            "run-control-unsupported: External Agent sessions support cancellation, not pause"
                .into(),
        );
    }
    repository::request_control(pool, &run_id, "pause").await
}

#[tauri::command]
pub async fn resume_run(app: tauri::AppHandle, run_id: String) -> Result<(), String> {
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
) -> Result<RunSummary, String> {
    retry::create_retry(crate::spaces::db::pool(&app).await?, &run_id, &step_ids).await
}

#[tauri::command]
pub async fn list_run_steps(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Vec<observation::StepSummary>, String> {
    observation::list_steps(crate::spaces::db::pool(&app).await?, &run_id).await
}

#[tauri::command]
pub async fn list_run_batch_items(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Vec<observation::BatchItemSummary>, String> {
    observation::list_batch_items(crate::spaces::db::pool(&app).await?, &run_id).await
}

#[tauri::command]
pub async fn list_run_events(
    app: tauri::AppHandle,
    run_id: String,
    after_sequence: i64,
) -> Result<Vec<observation::RunEvent>, String> {
    observation::list_events(
        crate::spaces::db::pool(&app).await?,
        &run_id,
        after_sequence,
    )
    .await
}

#[tauri::command]
pub async fn get_run_diagnostics(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<serde_json::Value, String> {
    observation::diagnostic_bundle(crate::spaces::db::pool(&app).await?, &run_id).await
}

#[tauri::command]
pub async fn submit_run_step_input(
    app: tauri::AppHandle,
    step_id: String,
    expected_version: i64,
    value: serde_json::Value,
) -> Result<(), String> {
    intervention::submit_input(
        crate::spaces::db::pool(&app).await?,
        &step_id,
        expected_version,
        &value,
    )
    .await
}

#[tauri::command]
pub async fn decide_run_step_review(
    app: tauri::AppHandle,
    step_id: String,
    expected_version: i64,
    approved: bool,
) -> Result<(), String> {
    intervention::decide_review(
        crate::spaces::db::pool(&app).await?,
        &step_id,
        expected_version,
        approved,
    )
    .await
}

#[tauri::command]
pub async fn reconcile_run_step(
    app: tauri::AppHandle,
    step_id: String,
    expected_version: i64,
    decision: intervention::ReconciliationDecision,
) -> Result<(), String> {
    intervention::reconcile(
        crate::spaces::db::pool(&app).await?,
        &step_id,
        expected_version,
        decision,
    )
    .await
}

#[tauri::command]
pub async fn continue_run_step(
    app: tauri::AppHandle,
    step_id: String,
    expected_version: i64,
) -> Result<(), String> {
    intervention::continue_debug_step(
        crate::spaces::db::pool(&app).await?,
        &step_id,
        expected_version,
    )
    .await
}
