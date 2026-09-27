use crate::command_error::CommandError;
use crate::spaces::db;
use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};
use ts_rs::TS;

use tauri::{AppHandle, Runtime};

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceNodeProjection {
    pub id: String,
    pub initial_x: f64,
    pub initial_y: f64,
    #[ts(type = "unknown")]
    pub data: Value,
}

pub(crate) async fn list_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<SpaceNodeProjection>, String> {
    let rows = sqlx::query(
        "SELECT id, initial_x, initial_y, data FROM spaces_nodes \
         WHERE space_id = ? AND deleted_at IS NULL ORDER BY created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(decode_node_projection).collect()
}

#[tauri::command]
pub async fn get_node<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_id: String,
) -> Result<SpaceNodeProjection, CommandError> {
    let row = sqlx::query(
        "SELECT id, initial_x, initial_y, data FROM spaces_nodes
         WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(db::pool(&app).await?)
    .await
    .map_err(db::sql_error)?
    .ok_or("node-unavailable: resource is unavailable")?;
    Ok((decode_node_projection(row))?)
}

fn decode_node_projection(row: sqlx::sqlite::SqliteRow) -> Result<SpaceNodeProjection, String> {
    let data_raw: String = row.try_get("data").map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&data_raw).map_err(db::json_error)?;
    Ok(SpaceNodeProjection {
        id: row.try_get::<String, _>("id").map_err(db::sql_error)?,
        initial_x: row.try_get::<f64, _>("initial_x").map_err(db::sql_error)?,
        initial_y: row.try_get::<f64, _>("initial_y").map_err(db::sql_error)?,
        data,
    })
}
