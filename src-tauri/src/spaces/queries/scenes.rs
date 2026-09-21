use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

#[tauri::command]
pub async fn list_scenes<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id,space_id,title,viewport_x,viewport_y,viewport_zoom,thumbnail_path,order_index,focus_structure_node_ids,\
         relation_kinds,relation_statuses,query_filter,created_at,updated_at \
         FROM spaces_scenes WHERE space_id=$1 ORDER BY order_index,created_at",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(row_to_json).collect()
}

fn row_to_json(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let parse = |name: &str| -> Result<Value, String> {
        let raw: String = row.try_get(name).map_err(db::sql_error)?;
        serde_json::from_str(&raw).map_err(db::json_error)
    };
    Ok(serde_json::json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?,
        "spaceId": row.try_get::<String,_>("space_id").map_err(db::sql_error)?,
        "title": row.try_get::<String,_>("title").map_err(db::sql_error)?,
        "viewportX": row.try_get::<f64,_>("viewport_x").map_err(db::sql_error)?,
        "viewportY": row.try_get::<f64,_>("viewport_y").map_err(db::sql_error)?,
        "viewportZoom": row.try_get::<f64,_>("viewport_zoom").map_err(db::sql_error)?,
        "thumbnailPath": row.try_get::<Option<String>,_>("thumbnail_path").map_err(db::sql_error)?,
        "orderIndex": row.try_get::<i64,_>("order_index").map_err(db::sql_error)?,
        "focusStructureNodeIds": parse("focus_structure_node_ids")?,
        "relationKinds": parse("relation_kinds")?,
        "relationStatuses": parse("relation_statuses")?,
        "queryFilter": parse("query_filter")?,
        "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))
}
