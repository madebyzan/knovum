use std::collections::HashSet;

use sqlx::{Sqlite, Transaction};

use super::model::KnowledgeChange;
use crate::spaces::db;

fn external_references_available(
    operations: &[KnowledgeChange],
    available_nodes: &HashSet<String>,
    available_relations: &HashSet<String>,
) -> bool {
    let temp_nodes: HashSet<&str> = operations.iter().filter_map(temp_node_id).collect();
    let temp_relations: HashSet<&str> = operations.iter().filter_map(temp_relation_id).collect();
    operations.iter().all(|operation| {
        references_available(
            operation,
            &temp_nodes,
            &temp_relations,
            available_nodes,
            available_relations,
        )
    })
}

fn temp_node_id(operation: &KnowledgeChange) -> Option<&str> {
    match operation {
        KnowledgeChange::CreateObject { temp_id, .. } => Some(temp_id),
        _ => None,
    }
}

fn temp_relation_id(operation: &KnowledgeChange) -> Option<&str> {
    match operation {
        KnowledgeChange::CreateRelation { temp_id, .. } => Some(temp_id),
        _ => None,
    }
}

fn references_available(
    operation: &KnowledgeChange,
    temp_nodes: &HashSet<&str>,
    temp_relations: &HashSet<&str>,
    nodes: &HashSet<String>,
    relations: &HashSet<String>,
) -> bool {
    let node_ok = |id: &str| temp_nodes.contains(id) || nodes.contains(id);
    let relation_ok = |id: &str| temp_relations.contains(id) || relations.contains(id);
    match operation {
        KnowledgeChange::CreateObject { .. } => true,
        KnowledgeChange::UpdateObject { node_id, .. } => node_ok(node_id),
        KnowledgeChange::CreateRelation {
            source_id,
            target_id,
            ..
        } => node_ok(source_id) && node_ok(target_id),
        KnowledgeChange::AddProvenance {
            target_kind,
            target_id,
            source_node_id,
            ..
        } => {
            node_ok(source_node_id)
                && if target_kind == "relation" {
                    relation_ok(target_id)
                } else {
                    node_ok(target_id)
                }
        }
    }
}

pub(crate) async fn validate_references(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    operations: &[KnowledgeChange],
) -> Result<(), String> {
    let nodes = load_ids(
        tx,
        "SELECT id FROM spaces_semantic_nodes WHERE space_id=$1",
        space_id,
    )
    .await?;
    let relations = load_ids(
        tx,
        "SELECT id FROM spaces_relations WHERE space_id=$1 AND status <> 'invalidated'",
        space_id,
    )
    .await?;
    if !external_references_available(operations, &nodes, &relations) {
        return Err(
            "Knowledge Change Set references an entity that is not available in the target Space"
                .into(),
        );
    }
    Ok(())
}

async fn load_ids(
    tx: &mut Transaction<'_, Sqlite>,
    query: &str,
    space_id: &str,
) -> Result<HashSet<String>, String> {
    Ok(sqlx::query_scalar(query)
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .into_iter()
        .collect())
}
