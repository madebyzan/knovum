use crate::command_error::CommandError;
mod contract;
mod jobs;
mod publish;
mod workflow;

use crate::media::acquisition::contracts::AcquisitionProgress;
pub(crate) use contract::ImportedMedia;

pub(crate) fn cleanup_stale_workspaces(app: &tauri::AppHandle) -> Result<(), String> {
    workflow::cleanup_stale(app)
}

#[tauri::command]
pub async fn import_media_from_url(
    app: tauri::AppHandle,
    job_id: String,
    url: String,
    on_progress: tauri::ipc::Channel<AcquisitionProgress>,
) -> Result<Option<ImportedMedia>, CommandError> {
    workflow::validate_source_url(&url)?;
    let tools = crate::media::acquisition::tools::resolve_required(&app)?;
    let (cancellation, _lease) = jobs::acquire(job_id.clone())?;
    if let Err(error) = on_progress.send(AcquisitionProgress {
        stage: "resolving",
        downloaded_bytes: 0,
        total_bytes: None,
        processed_ms: None,
        duration_ms: None,
    }) {
        return Err((error.to_string()).into());
    }
    let execution_id = job_id;
    Ok((crate::blocking::run(move || {
        workflow::run(&app, &execution_id, url, tools, &on_progress, &cancellation)
    })
    .await)?)
}

#[tauri::command]
pub async fn cancel_media_import(job_id: String) -> Result<(), CommandError> {
    Ok((jobs::cancel(&job_id))?)
}
