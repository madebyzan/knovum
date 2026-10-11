use crate::foundation::app_error::AppError;
use std::collections::HashSet;

use tauri::{AppHandle, Runtime};

use crate::spaces::{db, knowledge::invalidation};

#[tauri::command]
pub(crate) async fn revoke_relations<R: Runtime>(
    app: AppHandle<R>,
    relation_ids: Vec<String>,
) -> Result<(), AppError> {
    validate_relation_ids(&relation_ids)?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    for relation_id in relation_ids {
        if !invalidation::invalidate_relation(&mut tx, &relation_id, "human", None, now).await? {
            return Err((format!("Space Relation no longer exists: {relation_id}")).into());
        }
    }
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

#[tauri::command]
pub(crate) async fn restore_relations<R: Runtime>(
    app: AppHandle<R>,
    relation_ids: Vec<String>,
) -> Result<(), AppError> {
    validate_relation_ids(&relation_ids)?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    for relation_id in relation_ids {
        invalidation::restore_relation(&mut tx, &relation_id, "human", None, now).await?;
    }
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

fn validate_relation_ids(relation_ids: &[String]) -> Result<(), String> {
    if relation_ids.is_empty() {
        return Err("Relation selection must not be empty".into());
    }
    if relation_ids.len() > 100_000 {
        return Err("Relation selection exceeds the supported limit".into());
    }
    let mut unique = HashSet::with_capacity(relation_ids.len());
    for id in relation_ids {
        if id.is_empty() || id.encode_utf16().count() > 128 {
            return Err("Relation ID must contain 1 to 128 characters".into());
        }
        if !unique.insert(id) {
            return Err(format!("Duplicate Relation ID: {id}"));
        }
    }
    Ok(())
}
