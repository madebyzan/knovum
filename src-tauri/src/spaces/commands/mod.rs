use crate::command_error::CommandError;
pub(crate) mod catalog;
mod permanent_deletion;
pub(crate) mod relations;
pub(crate) mod retrieval;
pub(crate) mod scenes;
pub(crate) mod selection_transfer;

use tauri::{AppHandle, Runtime};

#[tauri::command]
pub async fn delete_space_permanently<R: Runtime>(
    app: AppHandle<R>,
    request: permanent_deletion::contracts::DeleteSpaceRequest,
) -> Result<Vec<String>, CommandError> {
    Ok((permanent_deletion::delete(&app, request).await)?)
}

#[tauri::command]
pub async fn move_structure_selection<R: Runtime>(
    app: AppHandle<R>,
    request: selection_transfer::contracts::TransferSelectionRequest,
) -> Result<selection_transfer::contracts::TransferSelectionResponse, CommandError> {
    selection_transfer::transfer_selection(&app, request)
        .await
        .map_err(CommandError::from_space_edit)
}
