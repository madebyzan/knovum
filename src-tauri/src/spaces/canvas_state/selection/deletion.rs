use tauri::{AppHandle, Runtime};

use super::contracts::{CanvasSelectionMutationResult, DeleteCanvasSelectionRequest};
use crate::spaces::{db, knowledge::invalidation, nodes};

pub(crate) async fn delete<R: Runtime>(
    app: AppHandle<R>,
    request: DeleteCanvasSelectionRequest,
) -> Result<CanvasSelectionMutationResult, String> {
    let pool = db::pool(&app).await?;
    let (result, deleted) = delete_in_transaction(pool, &request).await?;
    if !deleted.is_empty() {
        crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(&app, pool)
            .await;
    }
    Ok(result)
}

pub(super) async fn delete_in_transaction(
    pool: &sqlx::SqlitePool,
    request: &DeleteCanvasSelectionRequest,
) -> Result<(CanvasSelectionMutationResult, Vec<String>), String> {
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    nodes::update_many_in_transaction(
        &mut tx,
        &request.space_id,
        &request.survivor_changes,
        "human",
        now,
    )
    .await?;
    let deleted =
        nodes::trash_in_transaction(&mut tx, &request.space_id, &request.node_ids, now).await?;
    invalidate_selected_relations(&mut tx, &request.space_id, &request.relation_ids, now).await?;
    nodes::touch_updated_at(&mut tx, &request.space_id, now).await?;
    let revision = super::super::save_in_transaction(
        &mut tx,
        &request.space_id,
        request.base_revision,
        &request.document,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok((CanvasSelectionMutationResult { revision }, deleted))
}

async fn invalidate_selected_relations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    relation_ids: &[String],
    now: i64,
) -> Result<(), String> {
    for relation_id in relation_ids {
        let belongs_to_space = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM spaces_relations WHERE id=$1 AND space_id=$2 AND status <> 'invalidated')",
        )
        .bind(relation_id)
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if !belongs_to_space {
            return Err(format!(
                "Space relation {relation_id} is not active in this Space"
            ));
        }
        invalidation::invalidate_relation(tx, relation_id, "human", None, now).await?;
    }
    Ok(())
}
