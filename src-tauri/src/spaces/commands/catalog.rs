use crate::command_error::CommandError;
use serde::Deserialize;
use tauri::{AppHandle, Runtime};

use crate::spaces::{db, structure};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceInput {
    id: String,
    kind: String,
    title: String,
}

#[tauri::command]
pub async fn create_space<R: Runtime>(
    app: AppHandle<R>,
    space: SpaceInput,
) -> Result<(), CommandError> {
    if !matches!(space.kind.as_str(), "inbox" | "space") {
        return Err("Unsupported Space kind".into());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "INSERT INTO spaces(id, kind, title, archived_at, last_opened_at, created_at, updated_at) \
         VALUES (?, ?, ?, NULL, ?, ?, ?)",
    )
    .bind(&space.id)
    .bind(space.kind)
    .bind(space.title)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    structure::initialize_empty_in_transaction(&mut tx, &space.id).await?;
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

#[tauri::command]
pub async fn update_space_title<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    title: String,
) -> Result<(), CommandError> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let executed =
        sqlx::query("UPDATE spaces SET title = ?, updated_at = ? WHERE id = ? AND kind = 'space'")
            .bind(title)
            .bind(now)
            .bind(space_id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    Ok((db::require_one(executed, "Space no longer exists or cannot be renamed"))?)
}

#[tauri::command]
pub async fn archive_space<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<(), CommandError> {
    Ok((set_archived(&app, space_id, true).await)?)
}

#[tauri::command]
pub async fn unarchive_space<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<(), CommandError> {
    Ok((set_archived(&app, space_id, false).await)?)
}

#[tauri::command]
pub async fn mark_space_opened<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<(), CommandError> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let executed = sqlx::query("UPDATE spaces SET last_opened_at = ? WHERE id = ?")
        .bind(now)
        .bind(space_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    Ok((db::require_one(executed, "Space no longer exists"))?)
}

#[tauri::command]
pub async fn set_space_favorite<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    favorite: bool,
) -> Result<(), CommandError> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let executed = sqlx::query(
        "UPDATE spaces SET favorite = ?, updated_at = ? WHERE id = ? AND kind = 'space'",
    )
    .bind(favorite)
    .bind(now)
    .bind(space_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok((db::require_one(executed, "Space no longer exists or cannot be favorited"))?)
}

#[tauri::command]
pub async fn update_space_thumbnail<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    thumbnail_path: String,
) -> Result<Vec<String>, CommandError> {
    crate::spaces::assets::authorize_canvas_thumbnail(&app, &thumbnail_path)?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let previous =
        sqlx::query_scalar::<_, Option<String>>("SELECT thumbnail_path FROM spaces WHERE id=?")
            .bind(&space_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db::sql_error)?
            .ok_or("Space no longer exists")?;
    let executed = sqlx::query("UPDATE spaces SET thumbnail_path = ? WHERE id = ?")
        .bind(thumbnail_path)
        .bind(space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(executed, "Space no longer exists")?;
    tx.commit().await.map_err(db::sql_error)?;
    match crate::spaces::assets::unreferenced_paths(pool, previous.into_iter().collect()).await {
        Ok(paths) => Ok(paths),
        Err(error) => {
            log::warn!("Space thumbnail was saved, but obsolete-file cleanup could not be planned: {error}");
            Ok(Vec::new())
        }
    }
}

async fn set_archived<R: Runtime>(
    app: &AppHandle<R>,
    space_id: String,
    archived: bool,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let archived_at = archived.then_some(now);
    let pool = db::pool(app).await?;
    let executed = sqlx::query(
        "UPDATE spaces SET archived_at = ?, updated_at = ? WHERE id = ? AND kind = 'space'",
    )
    .bind(archived_at)
    .bind(now)
    .bind(space_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(executed, "Space no longer exists or cannot be archived")
}
