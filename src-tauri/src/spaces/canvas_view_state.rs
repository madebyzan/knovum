use crate::app_error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::db;

const MIN_ZOOM: f64 = 0.05;
const MAX_ZOOM: f64 = 16.0;
const MAX_POSITION: f64 = 1_000_000_000.0;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanvasViewState {
    x: f64,
    y: f64,
    zoom: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveCanvasViewStateRequest {
    space_id: String,
    view: CanvasViewState,
}

#[tauri::command]
pub(crate) async fn load_canvas_view_state<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Option<CanvasViewState>, AppError> {
    validate_space_id(&space_id)?;
    let pool = db::pool(&app).await?;
    let row = sqlx::query("SELECT x,y,zoom FROM spaces_canvas_view_state WHERE space_id=$1")
        .bind(space_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
    Ok((row
        .map(|row| -> Result<CanvasViewState, String> {
            let view = CanvasViewState {
                x: row.try_get("x").map_err(db::sql_error)?,
                y: row.try_get("y").map_err(db::sql_error)?,
                zoom: row.try_get("zoom").map_err(db::sql_error)?,
            };
            validate_view(view)?;
            Ok(view)
        })
        .transpose())?)
}

#[tauri::command]
pub(crate) async fn save_canvas_view_state<R: Runtime>(
    app: AppHandle<R>,
    request: SaveCanvasViewStateRequest,
) -> Result<(), AppError> {
    validate_space_id(&request.space_id)?;
    validate_view(request.view)?;
    let pool = db::pool(&app).await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces WHERE id=$1)")
        .bind(&request.space_id)
        .fetch_one(pool)
        .await
        .map_err(db::sql_error)?;
    if !exists {
        return Err("Space does not exist".into());
    }
    sqlx::query(
        "INSERT INTO spaces_canvas_view_state(space_id,x,y,zoom,updated_at) VALUES($1,$2,$3,$4,$5) \
         ON CONFLICT(space_id) DO UPDATE SET x=excluded.x,y=excluded.y,zoom=excluded.zoom,updated_at=excluded.updated_at",
    )
    .bind(request.space_id)
    .bind(request.view.x)
    .bind(request.view.y)
    .bind(request.view.zoom)
    .bind(crate::clock::unix_time_millis()?)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

fn validate_space_id(space_id: &str) -> Result<(), String> {
    if space_id.is_empty() || space_id.encode_utf16().count() > 128 {
        return Err("Space ID must contain 1 to 128 characters".into());
    }
    Ok(())
}

fn validate_view(view: CanvasViewState) -> Result<(), String> {
    if !view.x.is_finite()
        || !view.y.is_finite()
        || !view.zoom.is_finite()
        || view.x.abs() > MAX_POSITION
        || view.y.abs() > MAX_POSITION
        || !(MIN_ZOOM..=MAX_ZOOM).contains(&view.zoom)
    {
        return Err("Canvas view state is outside the supported range".into());
    }
    Ok(())
}
