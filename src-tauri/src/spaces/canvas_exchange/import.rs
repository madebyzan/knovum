use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::foundation::app_error::AppError;
use crate::spaces::{
    commands::relations::SpaceRelationInput,
    nodes::contracts::InsertNode,
    structure::{RelationStyle, StructureLayout, StructureNode},
};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ImportJsonCanvasNodesRequest {
    pub(super) space_id: String,
    pub(super) mutation_id: String,
    pub(super) base_revision: i64,
    pub(super) structure_nodes: Vec<StructureNode>,
    pub(super) layouts: Vec<StructureLayout>,
    pub(super) relation_styles: Vec<RelationStyle>,
    pub(super) nodes: Vec<JsonCanvasNodeImport>,
    pub(super) relations: Vec<SpaceRelationInput>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct JsonCanvasNodeImport {
    pub(super) source_id: String,
    pub(super) node: InsertNode,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ImportedJsonCanvasNode {
    pub(super) source_id: String,
    node_id: String,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct JsonCanvasImportReceipt {
    imported_nodes: Vec<ImportedJsonCanvasNode>,
    revision: i64,
}

pub(crate) async fn import_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: ImportJsonCanvasNodesRequest,
    digest: String,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<JsonCanvasImportReceipt, AppError> {
    super::import_control::check(cancellation)?;
    validate_source_ids(&request.nodes)?;
    let imported_nodes = request
        .nodes
        .iter()
        .map(|prepared_node| ImportedJsonCanvasNode {
            source_id: prepared_node.source_id.clone(),
            node_id: prepared_node.node.id.clone(),
        })
        .collect();
    let mut nodes = request
        .nodes
        .into_iter()
        .map(|prepared_node| prepared_node.node)
        .collect::<Vec<_>>();
    let pool = crate::spaces::db::pool(&app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    if let Some(import_receipt) =
        crate::spaces::structure::receipts::find_typed(&mut tx, &request.mutation_id, &digest)
            .await?
    {
        tx.commit().await.map_err(crate::spaces::db::sql_error)?;
        return Ok(import_receipt);
    }
    let (before_snapshot, current_revision) =
        crate::spaces::structure::repository::load_in_transaction(&mut tx, &request.space_id)
            .await?;
    if current_revision != request.base_revision {
        return Err(AppError::new(
            "space_edit_revision_conflict",
            "Space edit revision conflict",
        ));
    }
    let before =
        crate::spaces::structure::history::capture(&mut tx, &request.space_id, &before_snapshot)
            .await?;
    for node in &mut nodes {
        crate::spaces::knowledge::validation::normalize_node(
            &mut tx,
            &request.space_id,
            &node.kind,
            &mut node.data,
        )
        .await?;
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    crate::spaces::nodes::insert_rows_in_transaction(
        &mut tx,
        &request.space_id,
        &nodes,
        now,
        crate::spaces::assets::enrichment::jobs::intent::selection(&app)?.as_ref(),
    )
    .await?;
    for relation in &request.relations {
        if relation.space_id != request.space_id {
            return Err("Imported relation belongs to a different Space".into());
        }
        crate::spaces::commands::relations::create_relation_in_transaction(
            &mut tx, relation, "human", None, now,
        )
        .await?;
    }
    crate::spaces::nodes::touch_updated_at(&mut tx, &request.space_id, now).await?;
    let snapshot = crate::spaces::structure::repository::SpaceStructureSnapshot {
        structure: request.structure_nodes,
        layouts: request.layouts,
        relation_styles: request.relation_styles,
    };
    let revision = crate::spaces::structure::repository::save_in_transaction(
        &mut tx,
        &request.space_id,
        current_revision,
        &snapshot,
    )
    .await?;
    crate::spaces::structure::portal_integrity::validate_structure_targets(
        &mut tx,
        &[&request.space_id],
    )
    .await?;
    let after =
        crate::spaces::structure::history::capture(&mut tx, &request.space_id, &snapshot).await?;
    crate::spaces::structure::history::record(
        &mut tx,
        &request.space_id,
        &before,
        &after,
        None,
        None,
        revision,
    )
    .await?;
    let import_receipt = JsonCanvasImportReceipt {
        imported_nodes,
        revision,
    };
    crate::spaces::structure::receipts::insert_typed(
        &mut tx,
        &request.mutation_id,
        &request.space_id,
        &digest,
        revision,
        &[],
        &import_receipt,
    )
    .await?;
    super::import_control::check(cancellation)?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(import_receipt)
}

pub(super) fn validate_source_ids(nodes: &[JsonCanvasNodeImport]) -> Result<(), AppError> {
    let mut ids = HashSet::with_capacity(nodes.len());
    if nodes
        .iter()
        .any(|node| node.source_id.is_empty() || !ids.insert(node.source_id.as_str()))
    {
        return Err("JSON Canvas source node IDs must be non-empty and unique".into());
    }
    Ok(())
}
