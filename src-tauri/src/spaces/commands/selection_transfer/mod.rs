mod completion;
pub(crate) mod contracts;
mod operations;
mod validation;

use std::collections::HashSet;

use tauri::{AppHandle, Runtime};

use crate::foundation::app_error::AppError;
use crate::spaces::{db, knowledge, structure};
use contracts::TransferSelectionRequest;
use contracts::TransferSelectionResponse;
use structure::editor_protection::{EditorResourceProtection, SpaceEditorResource};

pub async fn transfer_selection<R: Runtime>(
    app: &AppHandle<R>,
    request: TransferSelectionRequest,
    editor_resources: &[SpaceEditorResource],
) -> Result<TransferSelectionResponse, AppError> {
    if request.source_space_id == request.target_space_id
        || request.root_structure_node_ids.is_empty()
    {
        return Err("Cross-Space move requires distinct Spaces and structure roots".into());
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let digest = structure::receipts::digest(&request)?;
    if let Some(transfer_receipt) =
        structure::receipts::find_typed(&mut tx, &request.mutation_id, &digest).await?
    {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(transfer_receipt);
    }
    let protection = EditorResourceProtection::capture(&mut tx, editor_resources).await?;
    validation::ensure_target_space(&mut tx, &request.target_space_id).await?;
    let (mut source_snapshot, source_revision) =
        structure::repository::load_in_transaction(&mut tx, &request.source_space_id).await?;
    let (mut target_snapshot, target_revision) =
        structure::repository::load_in_transaction(&mut tx, &request.target_space_id).await?;
    if source_revision != request.expected_source_edit_revision
        || target_revision != request.expected_target_edit_revision
    {
        return Err(AppError::new(
            "space_edit_revision_conflict",
            "Cross-Space edit revision conflict",
        ));
    }
    let source_before =
        structure::history::capture(&mut tx, &request.source_space_id, &source_snapshot).await?;
    let target_before =
        structure::history::capture(&mut tx, &request.target_space_id, &target_snapshot).await?;
    let structure_plan = structure::transfer::plan(
        &mut source_snapshot,
        &mut target_snapshot,
        &request.root_structure_node_ids,
        request.target_parent_id.as_deref(),
        request.target_order,
    )?;
    let selected = &structure_plan.moved_node_ids;
    let selected_set: HashSet<&str> = selected.iter().map(String::as_str).collect();
    let selected_structure_nodes: HashSet<&str> = structure_plan
        .moved_structure_node_ids
        .iter()
        .map(String::as_str)
        .collect();

    validation::validate_dependencies(&mut tx, &request.source_space_id, selected).await?;
    knowledge::grounding::references::detach_nodes(&mut tx, &request.source_space_id, selected)
        .await?;
    operations::update_portals(
        &mut tx,
        &request,
        &selected_set,
        &selected_structure_nodes,
        now,
    )
    .await?;
    let moved_relation_ids =
        operations::moved_relation_ids(&mut tx, &request.source_space_id, &selected_set).await?;
    let moved_provenance_ids =
        operations::moved_provenance_ids(&mut tx, &request.source_space_id, &selected_set).await?;

    structure::repository::rows::erase(
        &mut tx,
        &request.source_space_id,
        &structure_plan.source_structure_node_ids_deepest_first,
    )
    .await?;

    // Move nodes first. Database triggers enforce same-Space endpoints on dependent rows,
    // so dependents must only change Space after their referenced nodes have moved.
    operations::remove_scene_focus(
        &mut tx,
        &request.source_space_id,
        &selected_structure_nodes,
        now,
    )
    .await?;
    for id in selected {
        sqlx::query(
            "UPDATE spaces_nodes SET space_id=$1,updated_at=$2 WHERE id=$3 AND space_id=$4",
        )
        .bind(&request.target_space_id)
        .bind(now)
        .bind(id)
        .bind(&request.source_space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    }

    sqlx::query("UPDATE spaces_knowledge_derivation_inputs SET node_id=NULL WHERE node_id IN (SELECT value FROM json_each($1)) AND derivation_id IN (SELECT id FROM spaces_knowledge_derivations WHERE space_id=$2)")
        .bind(sqlx::types::Json(selected)).bind(&request.source_space_id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    for relation_id in &moved_relation_ids {
        sqlx::query("DELETE FROM spaces_relation_styles WHERE space_id=$1 AND relation_id=$2")
            .bind(&request.source_space_id)
            .bind(relation_id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    }
    sqlx::query(
        "UPDATE spaces_relations SET space_id=$1,updated_at=$2 WHERE space_id=$3 \
         AND source_id IN (SELECT value FROM json_each($4)) \
         AND target_id IN (SELECT value FROM json_each($4))",
    )
    .bind(&request.target_space_id)
    .bind(now)
    .bind(&request.source_space_id)
    .bind(sqlx::types::Json(selected))
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    operations::move_provenance(&mut tx, &request, &selected_set, now).await?;
    structure::transfer::move_relation_styles(
        &mut source_snapshot,
        &mut target_snapshot,
        &moved_relation_ids,
    );
    sqlx::query(
        "DELETE FROM spaces_chunk_embeddings WHERE chunk_id IN ( \
         SELECT id FROM spaces_knowledge_chunks WHERE node_id IN (SELECT value FROM json_each($1)) \
        )",
    )
    .bind(sqlx::types::Json(selected))
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_knowledge_chunks SET space_id=$1,updated_at=$2 WHERE space_id=$3 AND node_id IN (SELECT value FROM json_each($4))")
        .bind(&request.target_space_id).bind(now).bind(&request.source_space_id).bind(sqlx::types::Json(selected))
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    let transfer_receipt = completion::complete(
        &mut tx,
        completion::TransferCompletion {
            request: &request,
            digest: &digest,
            structure_plan,
            source_snapshot: &source_snapshot,
            target_snapshot: &target_snapshot,
            source_before: &source_before,
            target_before: &target_before,
            source_revision,
            target_revision,
            moved_relation_ids: &moved_relation_ids,
            moved_provenance_ids: &moved_provenance_ids,
            now,
        },
    )
    .await?;
    protection.ensure_preserved(&mut tx).await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::settle(app, pool).await;
    Ok(transfer_receipt)
}
