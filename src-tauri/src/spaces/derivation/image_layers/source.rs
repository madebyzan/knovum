use serde_json::Value;
use sqlx::Sqlite;
use tauri::AppHandle;

use super::contracts::SplitImageLayersRequest;
use crate::spaces::db;

/// The pinned source a split reads, as it stood when the split opened.
pub(super) struct SourceImage {
    pub(super) title: String,
    pub(super) asset_path: String,
}

/// Renders the pinned source, with its edits applied, into the PNG the local passes read. The
/// source is resolved from the request rather than from a derivation row, because a split opens
/// its derivations only once discovery has reported how many layers there are.
pub(super) async fn render_png(
    app: &AppHandle,
    source: &SourceImage,
    request: &SplitImageLayersRequest,
) -> Result<Vec<u8>, String> {
    Ok(super::super::source_snapshot::render_pinned_png(
        app,
        &source.asset_path,
        &request.source_revision,
        &request.source_edit,
    )
    .await?
    .png)
}

/// Reads the source a split is pinned to, confirming it still carries the revision and edit the
/// split opened against. Read once to open the split and again to settle it, so a source edited
/// while the models ran cannot publish layers of an image that no longer exists.
pub(super) async fn load<'e, E>(
    executor: E,
    request: &SplitImageLayersRequest,
) -> Result<SourceImage, String>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let encoded = sqlx::query_scalar::<_, String>(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL",
    )
    .bind(&request.source_node_id)
    .bind(&request.space_id)
    .fetch_optional(executor)
    .await
    .map_err(db::sql_error)?
    .ok_or("Image layer source is unavailable")?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if data.get("sourceRevision").and_then(Value::as_str) != Some(request.source_revision.as_str())
        || data.get("imageEdit").unwrap_or(&Value::Null) != &request.source_edit
    {
        return Err("Image source changed during layer splitting".into());
    }
    Ok(SourceImage {
        title: data
            .get("title")
            .and_then(Value::as_str)
            .ok_or("Image layer source title is unavailable")?
            .to_owned(),
        asset_path: data
            .get("assetPath")
            .and_then(Value::as_str)
            .ok_or("Image layer source path is unavailable")?
            .to_owned(),
    })
}
