use serde::Deserialize;
use super::contract::CanvasViewport;
use crate::spaces::db;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveViewportRequest { pub space_id: String, pub viewport: CanvasViewport }
pub(in crate::spaces) async fn read(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<CanvasViewport, String> {
    let value: Option<String> = sqlx::query_scalar("SELECT viewport FROM spaces_canvas_viewports WHERE space_id=$1").bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    let viewport = match value { Some(value) => serde_json::from_str(&value).map_err(db::json_error)?, None => CanvasViewport { x: 0.0, y: 0.0, zoom: 1.0 } };
    validate(&viewport)?; Ok(viewport)
}
pub(in crate::spaces) async fn save_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, viewport: &CanvasViewport) -> Result<(), String> {
    validate(viewport)?;
    sqlx::query("INSERT INTO spaces_canvas_viewports(space_id,viewport,updated_at) VALUES($1,$2,$3) ON CONFLICT(space_id) DO UPDATE SET viewport=excluded.viewport,updated_at=excluded.updated_at")
        .bind(space_id).bind(serde_json::to_string(viewport).map_err(db::json_error)?).bind(crate::clock::unix_time_millis()?)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
pub(super) async fn save(pool: &sqlx::SqlitePool, request: SaveViewportRequest) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    save_in_transaction(&mut tx, &request.space_id, &request.viewport).await?;
    tx.commit().await.map_err(db::sql_error)
}
fn validate(value: &CanvasViewport) -> Result<(), String> {
    if !value.x.is_finite() || !value.y.is_finite() || !value.zoom.is_finite() || value.zoom <= 0.0 { return Err("Invalid camera state".into()); } Ok(())
}
