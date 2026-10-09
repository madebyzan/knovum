pub(crate) mod contracts;
mod export;
pub(crate) mod import;
mod validation;

mod acquisition;
mod content;
mod conversion;
mod file_import;
mod import_control;

use crate::app_error::AppError;

pub(crate) fn reserve_maintenance() -> Result<crate::job_registry::JobAdmissionReservation, String>
{
    import_control::reserve_maintenance()
}

#[tauri::command]
pub(crate) fn cancel_json_canvas_import(mutation_id: String) -> Result<(), AppError> {
    Ok(import_control::cancel(&mutation_id)?)
}

#[tauri::command]
pub(crate) async fn export_json_canvas_file(
    app: tauri::AppHandle,
    request: export::ExportRequest,
) -> Result<(), AppError> {
    Ok(crate::blocking::run(move || export::write(&app, request)).await?)
}

#[tauri::command]
pub(crate) async fn import_json_canvas_file(
    app: tauri::AppHandle,
    request: file_import::ImportRequest,
    on_progress: tauri::ipc::Channel<crate::spaces::assets::import::commit::AssetImportProgress>,
) -> Result<import::JsonCanvasImportReceipt, AppError> {
    file_import::import(app, request, on_progress).await
}

#[cfg(test)]
mod tests;
