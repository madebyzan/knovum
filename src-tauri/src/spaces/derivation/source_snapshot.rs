use serde_json::Value;
use tauri::AppHandle;

use crate::spaces::db;

pub(super) async fn image_path(
    app: &AppHandle,
    derivation_id: &str,
    expected_edit: &Value,
) -> Result<String, String> {
    let pool = db::pool(app).await?;
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT n.data,i.source_revision FROM spaces_knowledge_derivations d JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' JOIN spaces_nodes n ON n.id=i.node_id AND n.space_id=d.space_id WHERE d.id=$1 AND d.status='running' AND n.kind='image' AND n.deleted_at IS NULL",
    )
    .bind(derivation_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let (encoded, revision) = row.ok_or("Image derivation source is unavailable")?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if data.get("sourceRevision").and_then(Value::as_str) != Some(revision.as_str())
        || data.get("imageEdit").unwrap_or(&Value::Null) != expected_edit
    {
        return Err("Image derivation source changed".into());
    }
    data.get("assetPath")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "Image derivation source asset is unavailable".into())
}

/// Renders a pinned source image, with its edits applied, into the PNG a derivation reads. The
/// content revision is verified around the render so a file replaced mid-render is refused rather
/// than silently derived from.
pub(super) async fn render_pinned_png(
    app: &AppHandle,
    asset_path: &str,
    source_revision: &str,
    edit: &Value,
) -> Result<crate::images::EditedImageOutput, String> {
    let app = app.clone();
    let asset_path = asset_path.to_owned();
    let source_revision = source_revision.to_owned();
    let edit = edit.clone();
    crate::foundation::blocking::run(move || {
        let source =
            crate::spaces::assets::authorize_spaces_asset(&app, std::path::Path::new(&asset_path))?;
        crate::file_inspection::verify_content_revision(&source, &source_revision)?;
        let rendered = crate::images::render_source_png(&source, &edit)?;
        crate::file_inspection::verify_content_revision(&source, &source_revision)?;
        Ok::<_, String>(rendered)
    })
    .await
}
