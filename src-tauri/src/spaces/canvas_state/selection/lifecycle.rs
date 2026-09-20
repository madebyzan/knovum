use tauri::{AppHandle, Runtime};

use super::contracts::{
    CanvasSelectionMutationResult, RestoreCanvasSelectionRequest,
    SetImportedCanvasNodesDeletedRequest,
};
use crate::spaces::{db, knowledge::invalidation, nodes};

pub(crate) async fn restore<R: Runtime>(
    app: AppHandle<R>,
    request: RestoreCanvasSelectionRequest,
) -> Result<CanvasSelectionMutationResult, String> {
    restore_in_transaction(db::pool(&app).await?, &request).await
}

pub(super) async fn restore_in_transaction(
    pool: &sqlx::SqlitePool,
    request: &RestoreCanvasSelectionRequest,
) -> Result<CanvasSelectionMutationResult, String> {
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    nodes::restore_in_transaction(&mut tx, &request.space_id, &request.node_ids, now).await?;
    nodes::update_many_in_transaction(
        &mut tx,
        &request.space_id,
        &request.survivor_changes,
        "human",
        now,
    )
    .await?;
    restore_selected_relations(&mut tx, &request.space_id, &request.relation_ids, now).await?;
    let revision = persist_document(
        &mut tx,
        &request.space_id,
        request.base_revision,
        &request.document,
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CanvasSelectionMutationResult { revision })
}

pub(crate) async fn set_imported_nodes_deleted<R: Runtime>(
    app: AppHandle<R>,
    request: SetImportedCanvasNodesDeletedRequest,
) -> Result<CanvasSelectionMutationResult, String> {
    let pool = db::pool(&app).await?;
    let result = set_imported_nodes_deleted_in_transaction(pool, &request).await?;
    if request.deleted {
        crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(&app, pool)
            .await;
    }
    Ok(result)
}

pub(super) async fn set_imported_nodes_deleted_in_transaction(
    pool: &sqlx::SqlitePool,
    request: &SetImportedCanvasNodesDeletedRequest,
) -> Result<CanvasSelectionMutationResult, String> {
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    let changed = if request.deleted {
        nodes::trash_in_transaction(&mut tx, &request.space_id, &request.node_ids, now).await?
    } else {
        nodes::restore_in_transaction(&mut tx, &request.space_id, &request.node_ids, now).await?
    };
    if changed.len() != request.node_ids.len() {
        return Err("Imported Canvas nodes are not in the required lifecycle state".into());
    }
    if !request.deleted {
        restore_selected_relations(&mut tx, &request.space_id, &request.relation_ids, now).await?;
    }
    let revision = persist_document(
        &mut tx,
        &request.space_id,
        request.base_revision,
        &request.document,
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CanvasSelectionMutationResult { revision })
}

async fn persist_document(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    base_revision: i64,
    document: &super::super::SpaceEditorDocument,
    now: i64,
) -> Result<i64, String> {
    nodes::touch_updated_at(tx, space_id, now).await?;
    super::super::save_in_transaction(tx, space_id, base_revision, document).await
}

async fn restore_selected_relations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    relation_ids: &[String],
    now: i64,
) -> Result<(), String> {
    for relation_id in relation_ids {
        let relation_space_id =
            invalidation::restore_relation(tx, relation_id, "human", None, now).await?;
        if relation_space_id != space_id {
            return Err(format!(
                "Space relation {relation_id} does not belong to this Space"
            ));
        }
    }
    Ok(())
}
