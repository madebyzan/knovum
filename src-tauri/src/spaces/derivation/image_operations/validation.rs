use serde_json::Value;

use super::contracts::DeriveImageRequest;
use crate::spaces::db;

pub fn request(request: &DeriveImageRequest) -> Result<(), String> {
    request.operation.validate()?;
    if request.id.trim().is_empty()
        || request.space_id.trim().is_empty()
        || request.source_revision.trim().is_empty()
    {
        return Err("Image operation identity is required".into());
    }
    if request.source_node_id == request.output_node_id {
        return Err("Image operation output must be a new node".into());
    }
    if !request.source_edit.is_null() && !request.source_edit.is_object() {
        return Err("Image operation source edit must be an object or null".into());
    }
    Ok(())
}

pub async fn source_edit(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &DeriveImageRequest,
) -> Result<String, String> {
    let data: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL",
    )
    .bind(&request.source_node_id)
    .bind(&request.space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Image operation source is unavailable")?;
    let data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    let current = data.get("imageEdit").cloned().unwrap_or(Value::Null);
    if current != request.source_edit {
        return Err("Image edits changed during image operation".into());
    }
    data.get("assetPath")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "Image operation source path is unavailable".into())
}
