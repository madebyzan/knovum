use serde_json::Value;
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::db;

use super::RunEnrichmentJobRequest;

pub(super) async fn load_current(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
) -> Result<Value, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query(
        "SELECT data FROM spaces_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL",
    )
    .bind(&request.space_id)
    .bind(&request.node_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Asset enrichment target no longer exists".to_owned())?;
    let raw: String = row.try_get("data").map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    let revision = data.get("sourceRevision").and_then(Value::as_str);
    if revision != Some(request.source_revision.as_str()) {
        return Err("Enrichment source revision is obsolete".to_owned());
    }
    Ok(data)
}

pub(super) async fn save(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    data: Value,
) -> Result<Value, String> {
    let field = match request.job_type {
        super::EnrichmentJobType::AnalyzeMedia => "mediaAnalysis",
        super::EnrichmentJobType::TranscribeMedia => "mediaKnowledge",
        super::EnrichmentJobType::AnalyzeImage
        | super::EnrichmentJobType::AnalyzeVideoFirstFrame => "visualKnowledge",
        _ => return Err("Enrichment job does not own node data".to_owned()),
    };
    let value = data
        .get(field)
        .cloned()
        .ok_or_else(|| format!("Enrichment did not produce node field {field}"))?;
    let fields = serde_json::Map::from_iter([(field.to_owned(), value)]);
    crate::spaces::assets::enrichment::commit::publish(
        app,
        &request.space_id,
        &request.node_id,
        Some(&request.source_revision),
        fields,
        None,
        Some(&request.job_id),
    )
    .await
}

pub(super) fn required_string<'a>(data: &'a Value, field: &str) -> Result<&'a str, String> {
    data.get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Enrichment requires node field {field}"))
}
