use crate::spaces::db;
use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};
use ts_rs::TS;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceSceneProjection {
    pub id: String,
    pub space_id: String,
    pub title: String,
    pub viewport_x: f64,
    pub viewport_y: f64,
    pub viewport_zoom: f64,
    pub thumbnail_path: Option<String>,
    pub order_index: i64,
    #[ts(type = "unknown")]
    pub focus_structure_node_ids: Value,
    #[ts(type = "unknown")]
    pub relation_kinds: Value,
    #[ts(type = "unknown")]
    pub relation_statuses: Value,
    #[ts(type = "unknown")]
    pub query_filter: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

pub(crate) async fn list_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<SpaceSceneProjection>, String> {
    let rows = sqlx::query(
        "SELECT id,space_id,title,viewport_x,viewport_y,viewport_zoom,thumbnail_path,order_index,focus_structure_node_ids,\
         relation_kinds,relation_statuses,query_filter,created_at,updated_at \
         FROM spaces_scenes WHERE space_id=$1 ORDER BY order_index,created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(decode_scene_projection).collect()
}

fn decode_scene_projection(row: sqlx::sqlite::SqliteRow) -> Result<SpaceSceneProjection, String> {
    let parse = |name: &str| -> Result<Value, String> {
        let raw: String = row.try_get(name).map_err(db::sql_error)?;
        serde_json::from_str(&raw).map_err(db::json_error)
    };
    Ok(SpaceSceneProjection {
        id: row.try_get::<String, _>("id").map_err(db::sql_error)?,
        space_id: row
            .try_get::<String, _>("space_id")
            .map_err(db::sql_error)?,
        title: row.try_get::<String, _>("title").map_err(db::sql_error)?,
        viewport_x: row.try_get::<f64, _>("viewport_x").map_err(db::sql_error)?,
        viewport_y: row.try_get::<f64, _>("viewport_y").map_err(db::sql_error)?,
        viewport_zoom: row
            .try_get::<f64, _>("viewport_zoom")
            .map_err(db::sql_error)?,
        thumbnail_path: row
            .try_get::<Option<String>, _>("thumbnail_path")
            .map_err(db::sql_error)?,
        order_index: row
            .try_get::<i64, _>("order_index")
            .map_err(db::sql_error)?,
        focus_structure_node_ids: parse("focus_structure_node_ids")?,
        relation_kinds: parse("relation_kinds")?,
        relation_statuses: parse("relation_statuses")?,
        query_filter: parse("query_filter")?,
        created_at: row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    })
}
