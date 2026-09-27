use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::{node_contracts, semantic_nodes};
use crate::spaces::knowledge::indexing::revision;

mod rules;
pub use rules::{
    validate_event_entity_kind, validate_event_pair, validate_provenance_quote,
    validate_relation_rationale, validate_text_limit, MAX_CHANGE_SET_OPERATIONS,
};

pub async fn validate_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    kind: &str,
    data: &Value,
) -> Result<Option<String>, String> {
    let object_type_id = node_contracts::validate_node(tx, space_id, node_id, kind, data).await?;
    // Hash only structurally bounded data. This keeps revision calculation behind the same
    // depth/size contract that protects every other durable Node validation path.
    revision::validate_declared(data)?;
    Ok(object_type_id)
}

/// User edits submit content; Rust computes the durable semantic revision after bounding the input.
pub async fn normalize_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    kind: &str,
    data: &mut Value,
) -> Result<Option<String>, String> {
    let object_type_id = node_contracts::validate_node(tx, space_id, node_id, kind, data).await?;
    revision::apply(data)?;
    Ok(object_type_id)
}

pub(in crate::spaces) async fn validate_same_space_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    target_id: &str,
) -> Result<(), String> {
    node_contracts::validate_same_space_node(tx, space_id, node_id, target_id).await
}

pub async fn validate_query_filter(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    filter: &Value,
) -> Result<(), String> {
    node_contracts::validate_query_filter(tx, space_id, filter).await
}

pub fn validate_object_status(status: &str) -> Result<(), String> {
    node_contracts::validate_object_status(status)
}

pub fn is_symmetric_relation_kind(kind: &str) -> bool {
    matches!(kind, "related" | "contradicts")
}

pub fn canonical_relation_endpoints(
    kind: &str,
    source_id: &str,
    target_id: &str,
) -> (String, String) {
    if is_symmetric_relation_kind(kind) && source_id > target_id {
        (target_id.to_owned(), source_id.to_owned())
    } else {
        (source_id.to_owned(), target_id.to_owned())
    }
}

pub fn validate_relation_kind(kind: &str) -> Result<(), String> {
    match kind {
        "reference" | "related" | "supports" | "contradicts" | "depends-on" | "supersedes"
        | "answers" | "uses" | "derives-from" | "caused-by" => Ok(()),
        _ => Err("Unsupported Space relation kind".into()),
    }
}

pub fn validate_relation_status(status: &str) -> Result<(), String> {
    match status {
        "active" | "contested" | "invalidated" => Ok(()),
        _ => Err("Unsupported Space relation status".into()),
    }
}

pub fn validate_current_relation_status(status: &str) -> Result<(), String> {
    match status {
        "active" | "contested" => Ok(()),
        "invalidated" => {
            Err("Invalidated Relations must use the dedicated lifecycle command".into())
        }
        _ => Err("Unsupported Space relation status".into()),
    }
}

pub async fn validate_relation_endpoints(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    source_id: &str,
    target_id: &str,
) -> Result<(), String> {
    if source_id == target_id {
        return Err("A relation cannot target itself".into());
    }
    validate_knowledge_node(tx, space_id, source_id, "Relation source").await?;
    validate_knowledge_node(tx, space_id, target_id, "Relation target").await
}

pub async fn validate_provenance_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    label: &str,
) -> Result<(), String> {
    validate_knowledge_node(tx, space_id, node_id, label).await
}

async fn validate_knowledge_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    label: &str,
) -> Result<(), String> {
    semantic_nodes::require(tx, space_id, node_id, label).await
}

pub fn validate_actor_kind(kind: &str) -> Result<(), String> {
    match kind {
        "human" | "assistant" | "agent" | "system" => Ok(()),
        _ => Err("Unsupported knowledge actor kind".into()),
    }
}

pub fn validate_confidence(value: Option<f64>) -> Result<(), String> {
    if value.is_some_and(|confidence| !(0.0..=1.0).contains(&confidence)) {
        return Err("Relation confidence must be between 0 and 1".into());
    }
    Ok(())
}

pub fn validate_provenance_target_kind(kind: &str) -> Result<(), String> {
    match kind {
        "node" | "relation" => Ok(()),
        _ => Err("Unsupported provenance target kind".into()),
    }
}

pub fn validate_provenance_status(status: &str) -> Result<(), String> {
    match status {
        "active" | "superseded" | "invalidated" => Ok(()),
        _ => Err("Unsupported provenance status".into()),
    }
}
