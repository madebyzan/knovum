use crate::foundation::app_error::AppError;
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
) -> Result<Vec<String>, AppError> {
    Ok((permanent_deletion::delete(&app, request).await)?)
}

#[tauri::command]
pub async fn move_structure_selection<R: Runtime>(
    app: AppHandle<R>,
    request: selection_transfer::contracts::TransferSelectionRequest,
    editor_resources: Vec<crate::spaces::structure::editor_protection::SpaceEditorResource>,
) -> Result<selection_transfer::contracts::TransferSelectionResponse, AppError> {
    selection_transfer::transfer_selection(&app, request, &editor_resources).await
}
