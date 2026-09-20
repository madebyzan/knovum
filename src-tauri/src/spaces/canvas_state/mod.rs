pub(in crate::spaces) mod contract;
mod appearance;
mod structure_contract;
mod gesture;
mod repository;
mod selection;
mod validation;

use tauri::{AppHandle, Runtime};

use crate::spaces::db;

pub(in crate::spaces) use contract::SpaceEditorDocument;
pub(in crate::spaces) use repository::save_in_transaction;
pub(in crate::spaces) use validation::validate_document;

use contract::{SaveCanvasStateRequest, StoredCanvasState};

#[tauri::command]
pub(crate) async fn load_canvas_state<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Option<StoredCanvasState>, String> {
    let pool = db::pool(&app).await?;
    Ok(repository::load(pool, &space_id)
        .await?
        .map(|(document, revision)| StoredCanvasState { document, revision }))
}

#[tauri::command]
pub(crate) async fn save_canvas_state<R: Runtime>(
    app: AppHandle<R>,
    request: SaveCanvasStateRequest,
) -> Result<i64, String> {
    let pool = db::pool(&app).await?;
    repository::save(
        pool,
        &request.space_id,
        request.base_revision,
        &request.document,
    )
    .await
}

#[tauri::command]
pub(crate) async fn commit_canvas_gesture<R: Runtime>(
    app: AppHandle<R>,
    request: gesture::CommitCanvasGestureRequest,
) -> Result<gesture::CommitCanvasGestureResult, String> {
    gesture::commit(app, request).await
}

#[tauri::command]
pub(crate) async fn delete_canvas_selection<R: Runtime>(
    app: AppHandle<R>,
    request: selection::DeleteCanvasSelectionRequest,
) -> Result<selection::CanvasSelectionMutationResult, String> {
    selection::delete(app, request).await
}

#[tauri::command]
pub(crate) async fn restore_canvas_selection<R: Runtime>(
    app: AppHandle<R>,
    request: selection::RestoreCanvasSelectionRequest,
) -> Result<selection::CanvasSelectionMutationResult, String> {
    selection::restore(app, request).await
}

#[tauri::command]
pub(crate) async fn set_imported_canvas_nodes_deleted<R: Runtime>(
    app: AppHandle<R>,
    request: selection::SetImportedCanvasNodesDeletedRequest,
) -> Result<selection::CanvasSelectionMutationResult, String> {
    selection::set_imported_nodes_deleted(app, request).await
}

#[cfg(test)]
mod tests;
