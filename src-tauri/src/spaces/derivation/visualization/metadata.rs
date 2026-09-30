use serde::Serialize;
use sqlx::Row;
use tauri::AppHandle;

use super::super::metadata_sources::{self, DerivationSource};
use crate::spaces::db;

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VisualizationMetadata {
    execution_source: crate::ai::tasks::ExecutionSource,
    intent: String,
    style: String,
    aspect_ratio: String,
    revised_prompt: Option<String>,
    created_at: i64,
    sources: Vec<DerivationSource>,
}

pub async fn get(app: &AppHandle, node_id: &str) -> Result<Option<VisualizationMetadata>, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query("SELECT d.id,d.created_at,g.execution_source,g.intent,g.style,g.aspect_ratio,g.revised_prompt FROM spaces_knowledge_derivations d JOIN spaces_generated_images g ON g.derivation_id=d.id WHERE d.output_node_id=$1 AND d.status='completed'")
        .bind(node_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    let id: String = row.try_get("id").map_err(db::sql_error)?;
    Ok(Some(VisualizationMetadata {
        execution_source: serde_json::from_str(
            &row.try_get::<String, _>("execution_source")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?,
        intent: row.try_get("intent").map_err(db::sql_error)?,
        style: row.try_get("style").map_err(db::sql_error)?,
        aspect_ratio: row.try_get("aspect_ratio").map_err(db::sql_error)?,
        revised_prompt: row.try_get("revised_prompt").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        sources: metadata_sources::load(pool, &id).await?,
    }))
}
