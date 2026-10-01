use crate::app_error::AppError;
use std::path::PathBuf;

use super::super::{enrichment, PreparedAsset};
use super::{commit, prepare, registry};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportSourceAccess {
    Scoped,
    Clipboard,
}

#[tauri::command]
pub async fn stage_asset_import(
    app: tauri::AppHandle,
    source_path: String,
    source_access: ImportSourceAccess,
    clipboard: tauri::State<'_, tauri_plugin_clipboard::Clipboard>,
) -> Result<prepare::PreparedSpaceImport, AppError> {
    let clipboard_paths = match source_access {
        ImportSourceAccess::Scoped => None,
        ImportSourceAccess::Clipboard => Some(clipboard.read_files()?),
    };
    Ok((crate::blocking::run(move || {
        let source = PathBuf::from(source_path);
        match clipboard_paths {
            Some(paths) => prepare::prepare_clipboard_asset(&app, source, &paths),
            None => prepare::prepare_asset(&app, source),
        }
    })
    .await)?)
}

#[tauri::command]
pub async fn commit_asset_import(
    job_id: String,
    on_progress: tauri::ipc::Channel<commit::AssetImportProgress>,
) -> Result<PreparedAsset, AppError> {
    crate::blocking::run(move || commit::copy_pending_import(&job_id, &on_progress))
        .await
        .map_err(enrichment::processing_error)
}

#[tauri::command]
pub async fn cancel_asset_import(job_id: String) -> Result<(), AppError> {
    Ok((registry::cancel(&job_id))?)
}

#[tauri::command]
pub async fn enrich_asset(
    app: tauri::AppHandle,
    space_id: String,
    node_id: String,
    job_id: String,
    on_update: tauri::ipc::Channel<serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    let (cancellation, _lease) = enrichment::registry::acquire(job_id)?;
    enrichment::enrich_node_with_cancellation(
        &app,
        space_id,
        node_id,
        &cancellation,
        Some(&on_update),
        None,
    )
    .await
}

#[tauri::command]
pub async fn cancel_asset_enrichment(job_id: String) -> Result<(), AppError> {
    Ok((enrichment::registry::cancel(&job_id))?)
}
