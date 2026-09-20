pub(crate) mod agent_runs;
mod contracts;
mod persistence;
mod queries;
mod validation;

use tauri::{AppHandle, Runtime};

pub use contracts::{SaveWorkflowRunRequest, WorkflowRunRecord};

#[tauri::command]
pub async fn save_creation_workflow_run<R: Runtime>(
    app: AppHandle<R>,
    request: SaveWorkflowRunRequest,
) -> Result<WorkflowRunRecord, String> {
    persistence::save(&app, request).await
}


#[tauri::command]
pub fn plan_creation_workflow_step(
    request: SaveWorkflowRunRequest,
) -> Result<Option<String>, String> {
    validation::next_step(&request)
}

pub(crate) async fn reconcile_interrupted(app: &AppHandle) -> Result<(), String> {
    let interrupted = agent_runs::reconcile_interrupted(app).await?;
    for run in interrupted {
        let workspace = crate::ai::capabilities::agent_artifacts::workspace(
            app,
            &run.space_id,
            &run.session_id,
        )
        .map_err(|error| error.to_string())?;
        crate::ai::capabilities::agent_artifacts::discard(&workspace)
            .map_err(|error| error.to_string())?;
    }
    persistence::reconcile_interrupted(app).await
}

#[cfg(test)]
mod tests;
