use crate::app_error::AppError;
pub(crate) mod capture;
mod execution;

pub(crate) use execution::execute_captured_step;

#[tauri::command]
pub async fn preview_workflow_capture(
    app: tauri::AppHandle,
    space_id: String,
    selected_node_ids: Vec<String>,
) -> Result<capture::CapturePreview, AppError> {
    Ok((capture::preview(
        crate::spaces::db::pool(&app).await?,
        &space_id,
        selected_node_ids,
    )
    .await)?)
}

#[tauri::command]
pub async fn save_captured_workflow(
    app: tauri::AppHandle,
    request: capture::SaveCapturedWorkflowRequest,
) -> Result<capture::SavedCapturedWorkflow, AppError> {
    Ok((capture::save(crate::spaces::db::pool(&app).await?, request).await)?)
}

#[tauri::command]
pub async fn list_captured_workflows(
    app: tauri::AppHandle,
) -> Result<capture::CapturedWorkflowLibrary, AppError> {
    Ok((capture::list(crate::spaces::db::pool(&app).await?).await)?)
}

#[tauri::command]
pub async fn rename_captured_workflow(
    app: tauri::AppHandle,
    workflow_id: String,
    expected_name: String,
    name: String,
) -> Result<capture::SavedCapturedWorkflow, AppError> {
    Ok((capture::rename(
        crate::spaces::db::pool(&app).await?,
        &workflow_id,
        &expected_name,
        &name,
    )
    .await)?)
}

#[tauri::command]
pub async fn revise_captured_workflow(
    app: tauri::AppHandle,
    request: capture::ReviseCapturedWorkflowRequest,
) -> Result<capture::SavedCapturedWorkflow, AppError> {
    Ok((capture::revise(crate::spaces::db::pool(&app).await?, request).await)?)
}

#[tauri::command]
pub async fn delete_captured_workflow(
    app: tauri::AppHandle,
    workflow_id: String,
    expected_name: String,
) -> Result<(), AppError> {
    Ok((capture::delete(
        crate::spaces::db::pool(&app).await?,
        &workflow_id,
        &expected_name,
    )
    .await)?)
}

#[tauri::command]
pub async fn start_captured_workflow(
    app: tauri::AppHandle,
    request: capture::StartCapturedWorkflowRequest,
) -> Result<crate::spaces::runs::RunObservation, AppError> {
    Ok((capture::start(&app, crate::spaces::db::pool(&app).await?, request).await)?)
}

#[tauri::command]
pub async fn list_captured_run_outputs(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Vec<capture::CapturedRunOutput>, AppError> {
    Ok((capture::list_run_outputs(crate::spaces::db::pool(&app).await?, &run_id).await)?)
}

#[tauri::command]
pub async fn list_captured_workflow_runs(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<capture::CapturedWorkflowRun>, AppError> {
    Ok((capture::list_run_history(crate::spaces::db::pool(&app).await?, &space_id).await)?)
}
