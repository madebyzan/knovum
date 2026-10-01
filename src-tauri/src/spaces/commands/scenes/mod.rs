use crate::app_error::AppError;
mod validation;

use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use self::validation::{json, validate_scene};
use crate::spaces::db;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceSceneInput {
    id: String,
    space_id: String,
    title: String,
    viewport_x: f64,
    viewport_y: f64,
    viewport_zoom: f64,
    thumbnail_path: Option<String>,
    order_index: i64,
    focus_structure_node_ids: Value,
    relation_kinds: Value,
    relation_statuses: Value,
    query_filter: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceSceneOrderInput {
    id: String,
    order_index: i64,
}

#[tauri::command]
pub async fn create_scene<R: Runtime>(
    app: AppHandle<R>,
    scene: SpaceSceneInput,
) -> Result<(), AppError> {
    validate_thumbnail_path(&app, scene.thumbnail_path.as_deref())?;
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    validate_scene(&mut tx, &scene).await?;
    sqlx::query(
        "INSERT INTO spaces_scenes(id,space_id,title,viewport_x,viewport_y,viewport_zoom,thumbnail_path,order_index,\
         focus_structure_node_ids,relation_kinds,relation_statuses,query_filter,created_at,updated_at) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$13)",
    )
    .bind(&scene.id)
    .bind(&scene.space_id)
    .bind(&scene.title)
    .bind(scene.viewport_x)
    .bind(scene.viewport_y)
    .bind(scene.viewport_zoom)
    .bind(&scene.thumbnail_path)
    .bind(scene.order_index)
    .bind(json(&scene.focus_structure_node_ids)?)
    .bind(json(&scene.relation_kinds)?)
    .bind(json(&scene.relation_statuses)?)
    .bind(json(&scene.query_filter)?)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

#[tauri::command]
pub async fn update_scene<R: Runtime>(
    app: AppHandle<R>,
    scene: SpaceSceneInput,
) -> Result<Vec<String>, AppError> {
    validate_thumbnail_path(&app, scene.thumbnail_path.as_deref())?;
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    validate_scene(&mut tx, &scene).await?;
    let previous = sqlx::query_scalar::<_, Option<String>>(
        "SELECT thumbnail_path FROM spaces_scenes WHERE id=$1 AND space_id=$2",
    )
    .bind(&scene.id)
    .bind(&scene.space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Space Saved View no longer exists")?;
    let executed = sqlx::query(
        "UPDATE spaces_scenes SET title=$1,viewport_x=$2,viewport_y=$3,viewport_zoom=$4,thumbnail_path=$5,order_index=$6,\
         focus_structure_node_ids=$7,relation_kinds=$8,relation_statuses=$9,query_filter=$10,updated_at=$11 \
         WHERE id=$12 AND space_id=$13",
    )
    .bind(&scene.title)
    .bind(scene.viewport_x)
    .bind(scene.viewport_y)
    .bind(scene.viewport_zoom)
    .bind(&scene.thumbnail_path)
    .bind(scene.order_index)
    .bind(json(&scene.focus_structure_node_ids)?)
    .bind(json(&scene.relation_kinds)?)
    .bind(json(&scene.relation_statuses)?)
    .bind(json(&scene.query_filter)?)
    .bind(now)
    .bind(&scene.id)
    .bind(&scene.space_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(executed, "Space Saved View no longer exists")?;
    tx.commit().await.map_err(db::sql_error)?;
    match crate::spaces::assets::unreferenced_paths(pool, previous.into_iter().collect()).await {
        Ok(paths) => Ok(paths),
        Err(error) => {
            log::warn!("Saved View was updated, but obsolete-thumbnail cleanup could not be planned: {error}");
            Ok(Vec::new())
        }
    }
}

fn validate_thumbnail_path<R: Runtime>(
    app: &AppHandle<R>,
    path: Option<&str>,
) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    crate::spaces::assets::authorize_canvas_thumbnail(app, path)
}

#[tauri::command]
pub async fn update_scene_orders<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    scenes: Vec<SpaceSceneOrderInput>,
) -> Result<(), AppError> {
    if scenes.is_empty() {
        return Ok(());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    for scene in scenes {
        let executed = sqlx::query(
            "UPDATE spaces_scenes SET order_index=$1,updated_at=$2 WHERE id=$3 AND space_id=$4",
        )
        .bind(scene.order_index)
        .bind(now)
        .bind(scene.id)
        .bind(&space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        db::require_one(executed, "Space Saved View order target no longer exists")?;
    }
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

#[tauri::command]
pub async fn delete_scene<R: Runtime>(
    app: AppHandle<R>,
    scene_id: String,
) -> Result<Vec<String>, AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let previous = sqlx::query_scalar::<_, Option<String>>(
        "SELECT thumbnail_path FROM spaces_scenes WHERE id=$1",
    )
    .bind(&scene_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Space Saved View no longer exists")?;
    let executed = sqlx::query("DELETE FROM spaces_scenes WHERE id=$1")
        .bind(scene_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(executed, "Space Saved View no longer exists")?;
    tx.commit().await.map_err(db::sql_error)?;
    match crate::spaces::assets::unreferenced_paths(pool, previous.into_iter().collect()).await {
        Ok(paths) => Ok(paths),
        Err(error) => {
            log::warn!(
                "Saved View was deleted, but thumbnail cleanup could not be planned: {error}"
            );
            Ok(Vec::new())
        }
    }
}
