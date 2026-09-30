use std::collections::HashMap;

use sqlx::{Sqlite, Transaction};

use crate::spaces::{
    knowledge::{contracts, relations},
    nodes::contracts::InsertNode,
};

#[derive(Default)]
pub(super) struct ApplyState {
    pub(super) node_ids: HashMap<String, String>,
    pub(super) relation_ids: HashMap<String, String>,
    pub(super) resolved_ids: HashMap<String, String>,
    pub(super) created_nodes: Vec<InsertNode>,
}

pub(super) async fn resolve_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    ids: &HashMap<String, String>,
    reference: &str,
) -> Result<String, String> {
    if let Some(id) = ids.get(reference) {
        return Ok(id.clone());
    }
    contracts::validate_provenance_node(
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
    ids: &HashMap<String, String>,
    reference: &str,
) -> Result<String, String> {
    if let Some(id) = ids.get(reference) {
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
