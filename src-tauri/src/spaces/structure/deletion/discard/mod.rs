use crate::foundation::app_error::AppError;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::spaces::{db, structure::StructureNode};

mod cleanup;

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct DiscardPendingContentRequest {
    pub(crate) space_id: String,
    pub(crate) node_id: String,
    pub(crate) expected_edit_revision: i64,
    pub(crate) mutation_id: String,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct DiscardPendingContentResponse {
    mutation_id: String,
    applied_at_revision: i64,
    asset_cleanup_candidates: Vec<String>,
}

pub(crate) async fn discard_pending_content<R: Runtime>(
    app: &AppHandle<R>,
    request: DiscardPendingContentRequest,
) -> Result<DiscardPendingContentResponse, AppError> {
    if request.mutation_id.is_empty() || request.mutation_id.encode_utf16().count() > 128 {
        return Err("Mutation ID must contain 1 to 128 characters".into());
    }
    if request.expected_edit_revision < 0 {
        return Err("Expected edit revision must be nonnegative".into());
    }
    let digest = super::super::receipts::digest(&request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(response) =
        super::super::receipts::find_typed(&mut tx, &request.mutation_id, &digest).await?
    {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(response);
    }
    let (mut snapshot, revision) =
        super::super::repository::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.expected_edit_revision {
        return Err(AppError::new(
            "space_edit_revision_conflict",
            "Space edit revision conflict",
        ));
    }
    let structure_node_id = content_structure_node_id(&snapshot.structure, &request.node_id)?;
    let index =
        super::super::planner::tree_policy::Index::new(&snapshot.structure, &snapshot.layouts);
    super::super::planner::tree_policy::require_editable(
        &snapshot.structure,
        &index,
        &structure_node_id,
    )?;
    let node_set = HashSet::from([request.node_id.as_str()]);
    super::super::portal_integrity::reject_node_targets(&mut tx, &request.space_id, &node_set)
        .await?;
    cleanup::reject_object_dependents(&mut tx, &request.space_id, &request.node_id).await?;
    let node_ids = HashSet::from([request.node_id.clone()]);
    let paths =
        crate::spaces::assets::referenced_paths_for_nodes(&mut tx, &request.space_id, &node_ids)
            .await?;
    let relations = cleanup::relation_ids(&mut tx, &request.space_id, &request.node_id).await?;
    let provenance =
        cleanup::provenance_ids(&mut tx, &request.space_id, &request.node_id, &relations).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::enqueue_nodes(
        &mut tx,
        &request.space_id,
        std::slice::from_ref(&request.node_id),
        now,
    )
    .await?;
    crate::spaces::knowledge::grounding::references::detach_nodes(
        &mut tx,
        &request.space_id,
        std::slice::from_ref(&request.node_id),
    )
    .await?;
    cleanup::delete_related(&mut tx, &relations, &provenance, &request.node_id).await?;
    snapshot
        .structure
        .retain(|structure_node| structure_node.id() != structure_node_id);
    snapshot
        .layouts
        .retain(|layout| layout.structure_node_id() != structure_node_id);
    snapshot
        .relation_styles
        .retain(|style| !relations.contains(&style.relation_id));
    normalize_orders(&mut snapshot.structure, &snapshot.layouts)?;
    super::super::repository::remove_transferred_structure_nodes_in_transaction(
        &mut tx,
        &request.space_id,
        &[structure_node_id],
    )
    .await?;
    sqlx::query("DELETE FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL")
        .bind(&request.node_id)
        .bind(&request.space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    super::super::repository::invalidate_history_for_nodes(
        &mut tx,
        &request.space_id,
        std::slice::from_ref(&request.node_id),
    )
    .await?;
    let applied = super::super::repository::save_in_transaction(
        &mut tx,
        &request.space_id,
        revision,
        &snapshot,
    )
    .await?;
    crate::spaces::nodes::touch_updated_at(&mut tx, &request.space_id, now).await?;
    let response = DiscardPendingContentResponse {
        mutation_id: request.mutation_id,
        applied_at_revision: applied,
        asset_cleanup_candidates: paths.into_iter().collect(),
    };
    super::super::receipts::insert_typed(
        &mut tx,
        &response.mutation_id,
        &request.space_id,
        &digest,
        applied,
        &[],
        &response,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::settle(app, pool).await;
    Ok(response)
}

fn content_structure_node_id(
    structure: &[StructureNode],
    node_id: &str,
) -> Result<String, AppError> {
    structure
        .iter()
        .find_map(|structure_node| match structure_node {
            StructureNode::Content {
                id,
                node_id: value,
                locked: false,
                ..
            } if value == node_id => Some(id.clone()),
            _ => None,
        })
        .ok_or_else(|| "Pending content structure_node not found or locked".into())
}

fn normalize_orders(
    structure: &mut [StructureNode],
    layouts: &[super::super::StructureLayout],
) -> Result<(), AppError> {
    let index = super::super::planner::tree_policy::Index::new(structure, layouts);
    Ok(super::super::planner::tree_policy::apply_sequences(
        structure,
        &index.children_by_parent_id,
    )?)
}
