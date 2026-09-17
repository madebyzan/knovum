use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::spaces::{
    canvas_state::CanvasDocumentV1, commands::relations::SpaceRelationInput,
    nodes::contracts::InsertNode,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ImportJsonCanvasNodesRequest {
    space_id: String,
    base_revision: i64,
    document: CanvasDocumentV1,
    nodes: Vec<JsonCanvasNodeImport>,
    relations: Vec<SpaceRelationInput>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct JsonCanvasNodeImport {
    source_id: String,
    node: InsertNode,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportedJsonCanvasNode {
    source_id: String,
    node_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportJsonCanvasResult {
    imported_nodes: Vec<ImportedJsonCanvasNode>,
    revision: i64,
}

pub(crate) async fn import_nodes<R: Runtime>(
    app: AppHandle<R>,
    request: ImportJsonCanvasNodesRequest,
) -> Result<ImportJsonCanvasResult, String> {
    validate_source_ids(&request.nodes)?;
    let imported_nodes = request
        .nodes
        .iter()
        .map(|item| ImportedJsonCanvasNode {
            source_id: item.source_id.clone(),
            node_id: item.node.id.clone(),
        })
        .collect();
    let nodes = request
        .nodes
        .into_iter()
        .map(|item| item.node)
        .collect::<Vec<_>>();
    let pool = crate::spaces::db::pool(&app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    crate::spaces::nodes::insert_many_in_transaction(&mut tx, &request.space_id, &nodes, now)
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
    let revision = crate::spaces::canvas_state::save_in_transaction(
        &mut tx,
        &request.space_id,
        request.base_revision,
        &request.document,
    )
    .await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(ImportJsonCanvasResult {
        imported_nodes,
        revision,
    })
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
