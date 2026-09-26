use crate::command_error::CommandError;
use tauri::{AppHandle, Runtime};

use super::edit_contract::{SpaceEditReceipt, SpaceEditSnapshotResponse};
use super::{receipts, repository};
use crate::spaces::db;

#[tauri::command]
pub(crate) async fn load_space_edit_snapshot<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<SpaceEditSnapshotResponse, CommandError> {
    validate_id("Space ID", &space_id)?;
    let pool = db::pool(&app).await?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let (snapshot, edit_revision) = repository::load_in_transaction(&mut tx, &space_id).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(SpaceEditSnapshotResponse {
        edit_revision,
        structure: snapshot.structure,
        layouts: snapshot.layouts,
        relation_styles: snapshot.relation_styles,
    })
}

#[tauri::command]
pub(crate) async fn get_space_edit_receipt_result<R: Runtime>(
    app: AppHandle<R>,
    mutation_id: String,
) -> Result<Option<serde_json::Value>, CommandError> {
    validate_id("Mutation ID", &mutation_id)?;
    Ok((receipts::result(db::pool(&app).await?, &mutation_id).await)?)
}

#[tauri::command]
pub(crate) async fn import_structure_clipboard<R: Runtime>(
    app: AppHandle<R>,
    request: super::clipboard::ImportStructureClipboardRequest,
) -> Result<super::clipboard::ImportStructureClipboardResponse, CommandError> {
    Ok((super::clipboard::import(&app, request).await)?)
}

#[tauri::command]
pub(crate) async fn list_recently_deleted_structure<R: Runtime>(
    app: AppHandle<R>,
    space_id: Option<String>,
) -> Result<Vec<super::deleted_commands::DeletedRootSummary>, CommandError> {
    Ok((super::deleted_commands::list(&app, space_id.as_deref()).await)?)
}

#[tauri::command]
pub(crate) async fn restore_deleted_structure<R: Runtime>(
    app: AppHandle<R>,
    request: super::deleted_commands::RestoreDeletedStructureRequest,
) -> Result<SpaceEditReceipt, CommandError> {
    Ok((super::deleted_commands::restore(&app, request).await)?)
}

#[tauri::command]
pub(crate) async fn permanently_delete_structure<R: Runtime>(
    app: AppHandle<R>,
    request: super::deleted_commands::PermanentlyDeleteStructureRequest,
) -> Result<Vec<String>, CommandError> {
    Ok((super::deleted_commands::permanently_delete(&app, request).await)?)
}

#[tauri::command]
pub(crate) async fn discard_pending_content<R: Runtime>(
    app: AppHandle<R>,
    request: super::deleted_commands::DiscardPendingContentRequest,
) -> Result<super::deleted_commands::DiscardPendingContentResponse, CommandError> {
    Ok((super::deleted_commands::discard_pending_content(&app, request).await)?)
}

fn validate_id(label: &str, id: &str) -> Result<(), String> {
    if id.is_empty() || id.encode_utf16().count() > 128 {
        return Err(format!("{label} must contain 1 to 128 characters"));
    }
    Ok(())
}
