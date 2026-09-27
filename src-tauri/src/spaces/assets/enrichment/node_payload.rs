use serde_json::Value;
use sqlx::Row;

use crate::spaces::db;

pub(super) async fn load_node_content(
    app: &tauri::AppHandle,
    space_id: &str,
    node_id: &str,
) -> Result<Value, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query(
        "SELECT data FROM spaces_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| format!("Space node {node_id} was not found"))?;
    let raw: String = row.try_get("data").map_err(db::sql_error)?;
    serde_json::from_str(&raw).map_err(db::json_error)
}

pub(super) fn required_string(data: &Value, key: &str) -> Result<String, String> {
    data.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("Space node {key} is required"))
}

pub(super) fn optional_string(data: &Value, key: &str) -> Option<String> {
    data.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub(super) fn set_optional(
    target: &mut serde_json::Map<String, Value>,
    key: &str,
    value: Option<&String>,
) {
    match value {
        Some(value) => {
            target.insert(key.to_owned(), Value::String(value.clone()));
        }
        None => {
            target.remove(key);
        }
    }
}
