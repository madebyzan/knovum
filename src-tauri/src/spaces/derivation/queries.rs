use serde::Serialize;
use sqlx::Row;
use tauri::AppHandle;

use super::{
    grounded_document, image_layers, image_operations, video_edit, video_generation, visualization,
};
use crate::app_error::AppError;
use crate::spaces::db;

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct DerivationFailure {
    pub action_kind: String,
    pub error: String,
    pub completed_at: i64,
}

#[tauri::command]
pub async fn get_grounded_document_metadata(
    app: AppHandle,
    node_id: String,
) -> Result<Option<grounded_document::metadata::GroundedDocumentMetadata>, AppError> {
    Ok((grounded_document::metadata::get(&app, &node_id).await)?)
}

#[tauri::command]
pub async fn get_visualization_metadata(
    app: AppHandle,
    node_id: String,
) -> Result<Option<visualization::metadata::VisualizationMetadata>, AppError> {
    Ok((visualization::metadata::get(&app, &node_id).await)?)
}

#[tauri::command]
pub async fn get_generated_video_metadata(
    app: AppHandle,
    node_id: String,
) -> Result<Option<video_generation::metadata::GeneratedVideoMetadata>, AppError> {
    Ok((video_generation::metadata::get(&app, &node_id).await)?)
}

#[tauri::command]
pub async fn get_derivation_failure(
    app: AppHandle,
    node_id: String,
) -> Result<Option<DerivationFailure>, AppError> {
    let pool = db::pool(&app).await?;
    let row = sqlx::query("SELECT action_kind,error,completed_at FROM spaces_knowledge_derivations WHERE output_node_id=$1 AND status='failed'")
        .bind(node_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    Ok(Some(DerivationFailure {
        action_kind: row.try_get("action_kind").map_err(db::sql_error)?,
        error: row.try_get("error").map_err(db::sql_error)?,
        completed_at: row.try_get("completed_at").map_err(db::sql_error)?,
    }))
}

#[tauri::command]
pub async fn get_image_layer_availability() -> image_layers::ImageLayerAvailability {
    image_layers::availability().await
}

#[tauri::command]
pub async fn get_image_operation_capabilities(
    app: AppHandle,
) -> image_operations::ImageOperationCapabilities {
    image_operations::capabilities(&app).await
}

#[tauri::command]
pub async fn list_video_fonts() -> Result<Vec<video_edit::fonts::InstalledFont>, AppError> {
    Ok(crate::blocking::run(|| Ok::<_, String>(video_edit::fonts::installed())).await?)
}
