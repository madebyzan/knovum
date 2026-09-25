pub(crate) mod catalog;
mod permanent_deletion;
pub(crate) mod relations;
pub(crate) mod retrieval;
pub(crate) mod scenes;
mod selection_transfer;

use tauri::{AppHandle, Runtime};

#[tauri::command]
pub async fn delete_space_permanently<R: Runtime>(
    app: AppHandle<R>,
    request: permanent_deletion::contracts::DeleteSpaceRequest,
) -> Result<Vec<String>, String> {
    permanent_deletion::delete(&app, request).await
}

#[tauri::command]
pub async fn move_structure_selection<R: Runtime>(
    app: AppHandle<R>,
    request: selection_transfer::contracts::TransferSelectionRequest,
) -> Result<selection_transfer::contracts::TransferSelectionResponse, String> {
    selection_transfer::transfer_selection(&app, request).await
}
