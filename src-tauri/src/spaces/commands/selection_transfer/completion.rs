use sqlx::{Sqlite, Transaction};

use super::contracts::{TransferSelectionRequest, TransferSelectionResponse};
use crate::foundation::app_error::AppError;
use crate::spaces::{db, knowledge::events, structure};

pub(super) struct TransferCompletion<'a> {
    pub request: &'a TransferSelectionRequest,
    pub digest: &'a str,
    pub structure_plan: structure::transfer::TransferPlan,
    pub source_snapshot: &'a mut structure::repository::SpaceStructureSnapshot,
    pub target_snapshot: &'a mut structure::repository::SpaceStructureSnapshot,
    pub source_before: &'a structure::history::HistoryState,
    pub target_before: &'a structure::history::HistoryState,
    pub source_revision: i64,
    pub target_revision: i64,
    pub moved_relation_ids: &'a [String],
    pub moved_provenance_ids: &'a [String],
    pub now: i64,
}

pub(super) async fn complete(
    tx: &mut Transaction<'_, Sqlite>,
    mut context: TransferCompletion<'_>,
) -> Result<TransferSelectionResponse, AppError> {
    let request = context.request;
    append_events(
        tx,
        &request.target_space_id,
        "node",
        &context.structure_plan.moved_node_ids,
        context.now,
    )
    .await?;
    append_events(
        tx,
        &request.target_space_id,
        "relation",
        context.moved_relation_ids,
        context.now,
    )
    .await?;
    append_events(
        tx,
        &request.target_space_id,
        "provenance",
        context.moved_provenance_ids,
        context.now,
    )
    .await?;
    for space_id in [&request.source_space_id, &request.target_space_id] {
        sqlx::query("UPDATE spaces SET updated_at=$1 WHERE id=$2")
            .bind(context.now)
            .bind(space_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    crate::spaces::knowledge::retrieval::invalidation_outbox::enqueue_nodes(
        tx,
        &request.source_space_id,
        &context.structure_plan.moved_node_ids,
        context.now,
    )
    .await?;
    let source_edit_revision = structure::repository::save_in_transaction(
        tx,
        &request.source_space_id,
        context.source_revision,
        context.source_snapshot,
    )
    .await?;
    let target_edit_revision = structure::repository::save_in_transaction(
        tx,
        &request.target_space_id,
        context.target_revision,
        context.target_snapshot,
    )
    .await?;
    structure::portal_integrity::validate_structure_targets(
        tx,
        &[&request.source_space_id, &request.target_space_id],
    )
    .await?;
    let source_after =
        structure::history::capture(tx, &request.source_space_id, context.source_snapshot).await?;
    let target_after =
        structure::history::capture(tx, &request.target_space_id, context.target_snapshot).await?;
    structure::history::record_group(
        tx,
        &uuid::Uuid::new_v4().to_string(),
        &[
            (
                &request.source_space_id,
                context.source_before,
                &source_after,
            ),
            (
                &request.target_space_id,
                context.target_before,
                &target_after,
            ),
        ],
    )
    .await?;
    let transfer_receipt = TransferSelectionResponse {
        moved_node_ids: context.structure_plan.moved_node_ids,
        moved_structure_node_ids: context.structure_plan.moved_structure_node_ids,
        source_edit_revision,
        target_edit_revision,
    };
    structure::receipts::insert_typed(
        tx,
        &request.mutation_id,
        &request.source_space_id,
        context.digest,
        source_edit_revision,
        &transfer_receipt.moved_structure_node_ids,
        &transfer_receipt,
    )
    .await?;
    Ok(transfer_receipt)
}

async fn append_events(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    entity_kind: &str,
    entity_ids: &[String],
    now: i64,
) -> Result<(), AppError> {
    let payload = serde_json::json!({});
    for id in entity_ids {
        events::append(
            tx,
            events::KnowledgeEvent::new(
                space_id,
                events::EventEntity::new(entity_kind, id),
                "moved",
                events::EventActor::new("human", None),
                &payload,
                now,
            ),
        )
        .await?;
    }
    Ok(())
}
