use super::{contracts::ImageSelection, local, source};
use crate::{app_error::AppError, local_resources::model_catalog::LocalModelId, spaces::db};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ImageSelectionPreviewRequest {
    pub id: String,
    pub space_id: String,
    pub node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
    pub model: LocalModelId,
    pub spec_id: String,
    pub selection: ImageSelection,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ImageSelectionPreview {
    pub mask_png_base64: String,
}

#[tauri::command]
pub(crate) async fn preview_image_selection(
    app: AppHandle,
    request: ImageSelectionPreviewRequest,
) -> Result<ImageSelectionPreview, AppError> {
    uuid::Uuid::parse_str(&request.id).map_err(|_| "Selection request ID must be a UUID")?;
    request.selection.validate()?;
    if request.model.image_spec()? != request.spec_id {
        return Err("Selection model specification changed".into());
    }
    let (cancellation, _lease) = crate::ai::executions::acquire(request.id.clone())?;
    let path = source_path(&app, &request).await?;
    let rendered = crate::blocking::run({
        let app = app.clone();
        let edit = request.source_edit.clone();
        let revision = request.source_revision.clone();
        move || {
            let path =
                crate::spaces::assets::authorize_spaces_asset(&app, std::path::Path::new(&path))?;
            crate::file_inspection::verify_content_revision(&path, &revision)?;
            let rendered = crate::images::render_source_png(&path, &edit)?;
            crate::file_inspection::verify_content_revision(&path, &revision)?;
            Ok::<_, String>(rendered)
        }
    })
    .await?;
    let source = source::from_rendered(rendered)?;
    let mask = local::preview(
        &app,
        request.model,
        &request.selection,
        &source,
        &request.id,
        &cancellation,
    )
    .await?;
    source_path(&app, &request).await?;
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled("Selection preview was cancelled"));
    }
    Ok(ImageSelectionPreview {
        mask_png_base64: mask,
    })
}

async fn source_path(
    app: &AppHandle,
    request: &ImageSelectionPreviewRequest,
) -> Result<String, AppError> {
    let encoded: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL")
        .bind(&request.node_id).bind(&request.space_id).fetch_optional(db::pool(app).await?).await.map_err(db::sql_error)?.ok_or("Selection source is unavailable")?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if data.get("sourceRevision").and_then(Value::as_str) != Some(&request.source_revision)
        || data.get("imageEdit").unwrap_or(&Value::Null) != &request.source_edit
    {
        return Err("Selection source changed".into());
    }
    Ok(data
        .get("assetPath")
        .and_then(Value::as_str)
        .ok_or("Selection asset is unavailable")?
        .into())
}
