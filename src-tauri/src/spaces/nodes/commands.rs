use tauri::{AppHandle, Runtime};

use super::contracts::{InsertNodesRequest, NodeSelectionRequest, UpdateNodesRequest};

#[tauri::command]
pub async fn create_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: InsertNodesRequest,
) -> Result<super::contracts::NodeEditorMutationResult, String> {
    super::insert_many(&app, request).await
}

#[tauri::command]
pub async fn update_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: UpdateNodesRequest,
) -> Result<(), String> {
    super::update_many(&app, request).await
}

#[tauri::command]
pub async fn trash_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: NodeSelectionRequest,
) -> Result<(), String> {
    super::set_deleted(&app, request, true).await
}

#[tauri::command]
pub async fn restore_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: NodeSelectionRequest,
) -> Result<(), String> {
    super::set_deleted(&app, request, false).await
}
