pub(in crate::spaces) mod changes;
mod contracts;
mod lifecycle_state;
mod archive;
mod receipts;
mod measurements;
mod execution;
mod lifecycle_edit;
mod result;
use tauri::{AppHandle, Emitter, Runtime};
use crate::spaces::{db, canvas_state, queries};
#[tauri::command]
pub(crate) async fn execute_space_edit<R: Runtime>(app: AppHandle<R>, request: contracts::ExecuteEditRequest) -> Result<contracts::EditResult, String> {
    let space_id = request.space_id.clone();
    let pool = db::pool(&app).await?;
    let result = execution::execute(pool, request).await?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(&app, pool).await;
    if let Err(error) = app.emit("spaces:editor-committed", serde_json::json!({"spaceId": space_id, "revision": result.revision, "mutationId": result.mutation_id})) { log::error!("Editor committed but notification failed: {error}"); }
    Ok(result)
}
#[tauri::command]
pub(crate) async fn query_space_edit_receipt<R: Runtime>(app: AppHandle<R>, request: contracts::ReceiptQuery) -> Result<Option<contracts::EditResult>, String> {
    let pool = db::pool(&app).await?; let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let receipt = receipts::find(&mut tx, &request.space_id, &request.mutation_id).await?;
    let result = match receipt { Some(receipt) => Some(result::current(&mut tx, &request.space_id, &receipt, true).await?), None => None };
    tx.commit().await.map_err(db::sql_error)?; Ok(result)
}
#[tauri::command]
pub(crate) async fn load_editor_workspace<R: Runtime>(app: AppHandle<R>, space_id: String) -> Result<contracts::EditorWorkspace, String> {
    let pool = db::pool(&app).await?; let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let (document, revision) = canvas_state::load_in_transaction(&mut tx, &space_id).await?;
    canvas_state::validation::validate_content_references(&mut tx, &space_id, &document).await?;
    let content = queries::read_editor_content(&mut tx, &space_id).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(contracts::EditorWorkspace { document, revision, content })
}
