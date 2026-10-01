use crate::command_error::CommandError;
pub(crate) mod contract;
mod export;
pub(crate) mod import;
mod validation;

mod acquisition;
mod content;
mod conversion;
mod file_import;
mod import_control;

#[tauri::command]
pub(crate) fn cancel_json_canvas_import(mutation_id: String) -> Result<(), CommandError> {
    Ok(import_control::cancel(&mutation_id)?)
}

#[tauri::command]
pub(crate) async fn export_json_canvas_file(
    app: tauri::AppHandle,
    request: export::ExportRequest,
) -> Result<(), CommandError> {
    Ok(crate::blocking::run(move || export::write(&app, request)).await?)
}

#[tauri::command]
pub(crate) async fn import_json_canvas_file(
    app: tauri::AppHandle,
    request: file_import::ImportRequest,
    on_progress: tauri::ipc::Channel<crate::spaces::assets::import::commit::AssetImportProgress>,
) -> Result<import::JsonCanvasImportReceipt, CommandError> {
    file_import::import(app, request, on_progress)
        .await
        .map_err(import_error)
}

fn import_error(error: String) -> CommandError {
    if error == import_control::CANCELLED {
        CommandError::new("json_canvas_import_cancelled", error)
    } else {
        CommandError::from_space_edit(error)
    }
}

#[cfg(test)]
mod tests;
