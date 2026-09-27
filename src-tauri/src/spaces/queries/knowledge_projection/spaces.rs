use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::db;

pub(crate) async fn list_spaces(
    app: &AppHandle,
    space_scope: Option<&str>,
) -> Result<Value, String> {
    let pool = db::pool(app).await?;
    let rows = if let Some(space_id) = space_scope {
        sqlx::query(
            "SELECT id,kind,title,last_opened_at,updated_at FROM spaces \
             WHERE id=$1 AND archived_at IS NULL",
        )
        .bind(space_id)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?
    } else {
        sqlx::query(
            "SELECT id,kind,title,last_opened_at,updated_at FROM spaces \
             WHERE archived_at IS NULL ORDER BY last_opened_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?
    };
    Ok(Value::Array(
        rows.into_iter()
            .map(|row| {
                Ok(json!({
                    "id": row.try_get::<String,_>("id").map_err(db::sql_error)?,
                    "kind": row.try_get::<String,_>("kind").map_err(db::sql_error)?,
                    "title": row.try_get::<String,_>("title").map_err(db::sql_error)?,
                    "lastOpenedAt": row.try_get::<i64,_>("last_opened_at").map_err(db::sql_error)?,
                    "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
                }))
            })
            .collect::<Result<Vec<_>, String>>()?,
    ))
}
