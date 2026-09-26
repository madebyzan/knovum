use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::spaces::{
    commands::relations::SpaceRelationInput,
    nodes::contracts::InsertNode,
    structure::{RelationStyle, StructureLayout, StructureNode},
};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ImportJsonCanvasNodesRequest {
    space_id: String,
    mutation_id: String,
    base_revision: i64,
    structure_nodes: Vec<StructureNode>,
    layouts: Vec<StructureLayout>,
    relation_styles: Vec<RelationStyle>,
    nodes: Vec<JsonCanvasNodeImport>,
    relations: Vec<SpaceRelationInput>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct JsonCanvasNodeImport {
    source_id: String,
    node: InsertNode,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ImportedJsonCanvasNode {
    source_id: String,
    node_id: String,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ImportJsonCanvasResult {
    imported_nodes: Vec<ImportedJsonCanvasNode>,
    revision: i64,
}

pub(crate) async fn import_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: ImportJsonCanvasNodesRequest,
) -> Result<ImportJsonCanvasResult, String> {
    validate_source_ids(&request.nodes)?;
    let digest = crate::spaces::structure::receipts::digest(&request)?;
    let imported_nodes = request
        .nodes
        .iter()
        .map(|prepared_node| ImportedJsonCanvasNode {
            source_id: prepared_node.source_id.clone(),
            node_id: prepared_node.node.id.clone(),
        })
        .collect();
    let nodes = request
        .nodes
        .into_iter()
        .map(|prepared_node| prepared_node.node)
        .collect::<Vec<_>>();
    let pool = crate::spaces::db::pool(&app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    if let Some(result) =
        crate::spaces::structure::receipts::find_typed(&mut tx, &request.mutation_id, &digest)
            .await?
    {
        tx.commit().await.map_err(crate::spaces::db::sql_error)?;
        return Ok(result);
    }
    let (before_snapshot, current_revision) =
        crate::spaces::structure::repository::load_in_transaction(&mut tx, &request.space_id)
            .await?;
    if current_revision != request.base_revision {
        return Err("space-edit-revision-conflict: Space edit revision conflict".into());
    }
    let before =
        crate::spaces::structure::history::capture(&mut tx, &request.space_id, &before_snapshot)
            .await?;
    let now = crate::clock::unix_time_millis()?;
    crate::spaces::nodes::insert_rows_in_transaction(&mut tx, &request.space_id, &nodes, now)
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
    crate::spaces::structure::portal_integrity::validate_structure_targets(&mut tx).await?;
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
    let result = ImportJsonCanvasResult {
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
        &result,
    )
    .await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(result)
}

pub(super) fn validate_source_ids(nodes: &[JsonCanvasNodeImport]) -> Result<(), String> {
    let mut ids = HashSet::with_capacity(nodes.len());
    if nodes
        .iter()
        .any(|node| node.source_id.is_empty() || !ids.insert(node.source_id.as_str()))
    {
        return Err("JSON Canvas source node IDs must be non-empty and unique".into());
    }
    Ok(())
}
