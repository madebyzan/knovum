use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use super::{assets, contracts::CreateImageLayersRequest};
use crate::spaces::db;

pub async fn data_uri(
    app: &AppHandle,
    request: &CreateImageLayersRequest,
) -> Result<String, String> {
    let path = super::super::source_snapshot::image_path(
        app,
        &super::persistence::derivation_id(&request.id, 0),
        &request.source_edit,
    )
    .await?;
    let app = app.clone();
    let edit = request.source_edit.clone();
    let rendered =
        crate::blocking::run(move || crate::images::render_source_png(&app, &path, &edit)).await?;
    assets::source_data_uri(&rendered.png)
}

pub async fn validate(
    tx: &mut Transaction<'_, Sqlite>,
    request: &CreateImageLayersRequest,
) -> Result<String, String> {
    let encoded = sqlx::query_scalar::<_, String>(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL",
    )
    .bind(&request.source_node_id)
    .bind(&request.space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Image layer source is unavailable")?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if data.get("sourceRevision").and_then(Value::as_str) != Some(request.source_revision.as_str())
        || data.get("imageEdit").unwrap_or(&Value::Null) != &request.source_edit
    {
        return Err("Image source changed during layer splitting".into());
    }
    data.get("assetPath")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "Image layer source path is unavailable".into())
}
