use serde::Serialize;
use sqlx::Row;
use tauri::AppHandle;

use super::super::metadata_sources::{self, DerivationSource};
use crate::spaces::db;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedVideoMetadata {
    provider: String,
    model: String,
    prompt: String,
    aspect_ratio: String,
    duration_seconds: i64,
    created_at: i64,
    sources: Vec<DerivationSource>,
}

pub async fn get(app: &AppHandle, node_id: &str) -> Result<Option<GeneratedVideoMetadata>, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query("SELECT d.id,d.created_at,g.provider,g.model,g.prompt,g.aspect_ratio,g.duration_seconds FROM spaces_knowledge_derivations d JOIN spaces_generated_videos g ON g.derivation_id=d.id WHERE d.output_node_id=$1 AND d.status='completed'")
        .bind(node_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    let derivation_id: String = row.try_get("id").map_err(db::sql_error)?;
    Ok(Some(GeneratedVideoMetadata {
        provider: row.try_get("provider").map_err(db::sql_error)?,
        model: row.try_get("model").map_err(db::sql_error)?,
        prompt: row.try_get("prompt").map_err(db::sql_error)?,
        aspect_ratio: row.try_get("aspect_ratio").map_err(db::sql_error)?,
        duration_seconds: row.try_get("duration_seconds").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        sources: metadata_sources::load(pool, &derivation_id).await?,
    }))
}
