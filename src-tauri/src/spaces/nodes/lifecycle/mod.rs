use tauri::{AppHandle, Runtime};

use super::{contracts::NodeSelectionRequest, space_timestamp::touch_updated_at};
use crate::spaces::{db, knowledge::retrieval::invalidation_outbox};

pub(crate) async fn set_deleted<R: Runtime>(
    app: &AppHandle<R>,
    request: NodeSelectionRequest,
    deleted: bool,
) -> Result<(), String> {
    if request.ids.is_empty() {
        return Ok(());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let ids =
        validation::active_unique_ids(&mut tx, &request.space_id, &request.ids, deleted).await?;
    if ids.is_empty() {
        return Ok(());
    }

    if deleted {
        mark_nodes_deleted(&mut tx, &request.space_id, &ids, now).await?;
        invalidation_outbox::enqueue_nodes(&mut tx, &request.space_id, &ids, now).await?;
    } else {
        restore_nodes(&mut tx, &request.space_id, &ids, now).await?;
    }
    touch_updated_at(&mut tx, &request.space_id, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    if deleted {
        invalidation_outbox::drain_and_report(app, pool).await;
    }
    Ok(())
}

pub(in crate::spaces) async fn trash_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    requested_ids: &[String],
    now: i64,
) -> Result<Vec<String>, String> {
    let ids = validation::active_unique_ids(tx, space_id, requested_ids, true).await?;
    if !ids.is_empty() {
        mark_nodes_deleted(tx, space_id, &ids, now).await?;
        invalidation_outbox::enqueue_nodes(tx, space_id, &ids, now).await?;
    }
    Ok(ids)
}

pub(in crate::spaces) async fn restore_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    requested_ids: &[String],
    now: i64,
) -> Result<Vec<String>, String> {
    let ids = validation::active_unique_ids(tx, space_id, requested_ids, false).await?;
    if !ids.is_empty() {
        restore_nodes(tx, space_id, &ids, now).await?;
    }
    Ok(ids)
}

mod effects;
mod validation;
use effects::{mark_nodes_deleted, restore_nodes};
