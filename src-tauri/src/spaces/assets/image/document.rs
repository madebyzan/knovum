use serde_json::Value;

use crate::spaces::db;

pub(in crate::spaces::assets) struct ImageDocument {
    pub source_path: String,
    pub source_revision: String,
    pub file_name: String,
    pub edit: Value,
}

pub(in crate::spaces::assets) async fn load(
    app: &tauri::AppHandle,
    space_id: &str,
    node_id: &str,
) -> Result<ImageDocument, String> {
    let pool = db::pool(app).await?;
    let data: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Image source is unavailable")?;
    let data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    from_payload(&data)
}

pub(in crate::spaces::assets) fn from_payload(data: &Value) -> Result<ImageDocument, String> {
    Ok(ImageDocument {
        source_path: data
            .get("assetPath")
            .and_then(Value::as_str)
            .ok_or("Image source path is unavailable")?
            .to_owned(),
        source_revision: data
            .get("sourceRevision")
            .and_then(Value::as_str)
            .ok_or("Image source revision is unavailable")?
            .to_owned(),
        file_name: data
            .get("fileName")
            .and_then(Value::as_str)
            .ok_or("Image file name is unavailable")?
            .to_owned(),
        edit: data.get("imageEdit").cloned().unwrap_or(Value::Null),
    })
}
