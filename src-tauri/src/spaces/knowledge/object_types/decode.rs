use serde_json::Value;
use sqlx::Row;

use crate::spaces::db;

pub(super) fn decode_object_type(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let fields: String = row.try_get("fields").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?,
        "spaceId": row.try_get::<Option<String>,_>("space_id").map_err(db::sql_error)?,
        "name": row.try_get::<String,_>("name").map_err(db::sql_error)?,
        "icon": row.try_get::<String,_>("icon").map_err(db::sql_error)?,
        "description": row.try_get::<String,_>("description").map_err(db::sql_error)?,
        "fields": serde_json::from_str::<Value>(&fields).map_err(db::json_error)?,
        "system": row.try_get::<i64,_>("system").map_err(db::sql_error)? == 1,
        "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))
}
