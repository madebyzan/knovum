use crate::command_error::CommandError;
use tauri::{AppHandle, Runtime};

use super::contracts::{InsertNodesRequest, UpdateNodesRequest};

#[tauri::command]
pub async fn create_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: InsertNodesRequest,
) -> Result<i64, CommandError> {
    Ok((super::insert_many(&app, request).await)?)
}

#[tauri::command]
pub(crate) async fn update_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: UpdateNodesRequest,
) -> Result<super::contracts::UpdateNodesResponse, CommandError> {
    Ok((super::update_many(&app, request).await)?)
}
