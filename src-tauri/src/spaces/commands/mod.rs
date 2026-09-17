pub(crate) mod catalog;
pub(crate) mod grounded;
mod permanent_deletion;
pub(crate) mod relations;
pub(crate) mod retrieval;
pub(crate) mod scenes;
mod selection_transfer;

use tauri::{AppHandle, Runtime};

#[tauri::command]
pub async fn delete_deleted_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: permanent_deletion::contracts::DeleteDeletedNodesRequest,
) -> Result<Vec<String>, String> {
    permanent_deletion::delete_nodes(&app, request).await
}

#[tauri::command]
pub async fn delete_space_permanently<R: Runtime>(
    app: AppHandle<R>,
    request: permanent_deletion::contracts::DeleteSpaceRequest,
) -> Result<Vec<String>, String> {
    permanent_deletion::delete(&app, request).await
}

#[tauri::command]
pub async fn move_space_selection<R: Runtime>(
    app: AppHandle<R>,
    request: selection_transfer::contracts::TransferSelectionRequest,
) -> Result<Vec<String>, String> {
    selection_transfer::transfer_selection(&app, request).await
}

pub(crate) fn dedup_and_validate_node_ids(
    ids: &[String],
    empty_error_msg: &str,
) -> Result<Vec<String>, String> {
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        if id.trim().is_empty() {
            return Err(empty_error_msg.into());
        }
        if seen.insert(id.as_str()) {
            unique.push(id.clone());
        }
    }
    Ok(unique)
}
