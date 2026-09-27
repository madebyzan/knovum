use super::edit_contract::SpaceEditSnapshotResponse;
use super::{receipts, repository, validation};
use crate::command_error::CommandError;
use crate::spaces::db;
use tauri::{AppHandle, Runtime};

#[tauri::command]
pub(crate) async fn load_space_edit_snapshot<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<SpaceEditSnapshotResponse, CommandError> {
    validation::validate_id("Space ID", &space_id)?;
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
pub(crate) async fn get_space_edit_receipt<R: Runtime>(
    app: AppHandle<R>,
    mutation_id: String,
) -> Result<Option<serde_json::Value>, CommandError> {
    validation::validate_id("Mutation ID", &mutation_id)?;
    Ok((receipts::load_committed_projection(db::pool(&app).await?, &mutation_id).await)?)
}

#[tauri::command]
pub(crate) async fn list_recently_deleted_structure<R: Runtime>(
    app: AppHandle<R>,
    space_id: Option<String>,
) -> Result<Vec<super::deletion::DeletedStructureRoot>, CommandError> {
    Ok((super::deletion::list(&app, space_id.as_deref()).await)?)
}
