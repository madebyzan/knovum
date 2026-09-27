mod boundaries;
mod candidate;
mod fingerprint;
mod inspect;
pub(crate) mod library;
mod lookup;
mod management;
mod order;
mod run;
mod selection;
pub(crate) mod types;

use sqlx::{Sqlite, SqlitePool, Transaction};

pub use self::types::CapturePreview;
use crate::spaces::db;
pub(crate) use library::{
    CapturedWorkflowLibrary, SaveCapturedWorkflowRequest, SavedCapturedWorkflow,
};
pub(crate) use management::ReviseCapturedWorkflowRequest;
pub(crate) use run::CapturedRunOutput;
pub(crate) use run::CapturedWorkflowRun;
pub(crate) use run::StartCapturedWorkflowRequest;

pub async fn preview(
    pool: &SqlitePool,
    space_id: &str,
    selected_node_ids: Vec<String>,
) -> Result<CapturePreview, String> {
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let captured_workflow = in_transaction(&mut tx, space_id, selected_node_ids).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(captured_workflow)
}

pub(crate) async fn in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    selected_node_ids: Vec<String>,
) -> Result<CapturePreview, String> {
    inspect::in_transaction(tx, space_id, selected_node_ids).await
}

pub(crate) async fn save(
    pool: &SqlitePool,
    request: SaveCapturedWorkflowRequest,
) -> Result<SavedCapturedWorkflow, String> {
    library::save(pool, request).await
}

pub(crate) async fn list(pool: &SqlitePool) -> Result<CapturedWorkflowLibrary, String> {
    library::list(pool).await
}

pub(crate) async fn rename(
    pool: &SqlitePool,
    workflow_id: &str,
    expected_name: &str,
    name: &str,
) -> Result<SavedCapturedWorkflow, String> {
    management::rename(pool, workflow_id, expected_name, name).await
}

pub(crate) async fn delete(
    pool: &SqlitePool,
    workflow_id: &str,
    expected_name: &str,
) -> Result<(), String> {
    management::delete(pool, workflow_id, expected_name).await
}

pub(crate) async fn revise(
    pool: &SqlitePool,
    request: ReviseCapturedWorkflowRequest,
) -> Result<SavedCapturedWorkflow, String> {
    management::revise(pool, request).await
}

pub(crate) async fn start(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: StartCapturedWorkflowRequest,
) -> Result<crate::spaces::runs::RunObservation, String> {
    run::start(app, pool, request).await
}

pub(crate) async fn execute_step(
    pool: &SqlitePool,
    claim: &crate::spaces::runs::AttemptClaim,
) -> Result<crate::spaces::runs::StepOutcome, String> {
    run::execute_step(pool, claim).await
}

pub(crate) async fn list_run_outputs(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<CapturedRunOutput>, String> {
    run::list_outputs(pool, run_id).await
}

pub(crate) async fn list_run_history(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<CapturedWorkflowRun>, String> {
    run::list_history(pool, space_id).await
}

#[cfg(test)]
mod tests;
