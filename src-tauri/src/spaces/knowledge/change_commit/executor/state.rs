use std::collections::HashMap;

use sqlx::{Sqlite, Transaction};

use crate::spaces::{
    knowledge::{relations, semantic_nodes},
    nodes::contracts::InsertNode,
};

#[derive(Default)]
pub(super) struct ApplyState {
    pub(super) node_id_by_temp_id: HashMap<String, String>,
    pub(super) relation_id_by_temp_id: HashMap<String, String>,
    pub(super) resolved_id_by_temp_id: HashMap<String, String>,
    pub(super) created_nodes: Vec<InsertNode>,
}

pub(super) async fn resolve_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id_by_temp_id: &HashMap<String, String>,
    reference: &str,
) -> Result<String, String> {
    if let Some(id) = node_id_by_temp_id.get(reference) {
        return Ok(id.clone());
    }
    semantic_nodes::require(
        tx,
        space_id,
        reference,
        "Knowledge Change Set node reference",
    )
    .await?;
    Ok(reference.to_owned())
}

pub(super) async fn resolve_relation(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    relation_id_by_temp_id: &HashMap<String, String>,
    reference: &str,
) -> Result<String, String> {
    if let Some(id) = relation_id_by_temp_id.get(reference) {
        return Ok(id.clone());
    }
    if !relations::exists(tx, space_id, reference).await? {
        return Err(
            "Knowledge Change Set relation reference must be an active Relation in this Space"
                .into(),
        );
    }
    Ok(reference.to_owned())
}
