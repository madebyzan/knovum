use serde::Serialize;
use sqlx::Row;
use tauri::AppHandle;

use super::{grounded_document, knowledge_video, visualization};
use crate::spaces::db;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivationFailure {
    pub action_kind: String,
    pub error: String,
    pub completed_at: i64,
}

#[tauri::command]
pub async fn get_grounded_document_metadata(
    app: AppHandle,
    node_id: String,
) -> Result<Option<grounded_document::metadata::GroundedDocumentMetadata>, String> {
    grounded_document::metadata::get(&app, &node_id).await
}

#[tauri::command]
pub async fn get_visualization_metadata(
    app: AppHandle,
    node_id: String,
) -> Result<Option<visualization::metadata::VisualizationMetadata>, String> {
    visualization::metadata::get(&app, &node_id).await
}

#[tauri::command]
pub async fn get_generated_video_metadata(
    app: AppHandle,
    node_id: String,
) -> Result<Option<knowledge_video::metadata::GeneratedVideoMetadata>, String> {
    knowledge_video::metadata::get(&app, &node_id).await
}

#[tauri::command]
pub async fn get_derivation_failure(
    app: AppHandle,
    node_id: String,
) -> Result<Option<DerivationFailure>, String> {
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
