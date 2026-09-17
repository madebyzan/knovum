use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use tauri::AppHandle;

use super::super::metadata_sources::{self, DerivationSource};
use crate::spaces::db;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundedDocumentMetadata {
    action_kind: String,
    provider: String,
    model: String,
    created_at: i64,
    citations: Value,
    sources: Vec<DerivationSource>,
}

pub async fn get(
    app: &AppHandle,
    node_id: &str,
) -> Result<Option<GroundedDocumentMetadata>, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query("SELECT d.id,d.action_kind,d.created_at,g.provider,g.model,g.citations FROM spaces_knowledge_derivations d JOIN spaces_grounded_documents g ON g.derivation_id=d.id WHERE d.output_node_id=$1 AND d.status='completed'")
        .bind(node_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    let derivation_id: String = row.try_get("id").map_err(db::sql_error)?;
    let sources = metadata_sources::load(pool, &derivation_id).await?;
    let citations: String = row.try_get("citations").map_err(db::sql_error)?;
    Ok(Some(GroundedDocumentMetadata {
        action_kind: row.try_get("action_kind").map_err(db::sql_error)?,
        provider: row.try_get("provider").map_err(db::sql_error)?,
        model: row.try_get("model").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        citations: serde_json::from_str(&citations).map_err(db::json_error)?,
        sources,
    }))
}
