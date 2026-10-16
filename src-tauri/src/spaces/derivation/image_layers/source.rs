use serde_json::Value;
use sqlx::Sqlite;
use tauri::AppHandle;

use super::contracts::SplitImageLayersRequest;
use crate::foundation::app_error::AppError;
use crate::spaces::db;

/// The pinned source a split reads, as it stood when the split opened.
pub(super) struct SourceImage {
    pub(super) title: String,
    pub(super) asset_path: String,
}

/// Renders the source image, with its edits applied, into the PNG the local passes read.
pub(super) async fn load_png(
    app: &AppHandle,
    scratch_key: &str,
    edit: &Value,
) -> Result<Vec<u8>, String> {
    let path = super::super::source_snapshot::image_path(app, scratch_key, edit).await?;
    let app = app.clone();
    let edit = edit.clone();
    let rendered = crate::foundation::blocking::run(move || {
        let source =
            crate::spaces::assets::authorize_spaces_asset(&app, std::path::Path::new(&path))?;
        crate::images::render_source_png(&source, &edit)
    })
    .await?;
    Ok(rendered.png)
}

pub(super) async fn describe(
    app: &AppHandle,
    request: &SplitImageLayersRequest,
) -> Result<SourceImage, AppError> {
    Ok(validate(db::pool(app).await?, request).await?)
}

/// Confirms the source still carries the pinned revision and edit the split opened against.
pub(super) async fn validate<'e, E>(
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
