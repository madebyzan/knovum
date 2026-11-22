use std::collections::HashMap;

use sqlx::{Sqlite, Transaction};

use crate::spaces::{
    knowledge::{
        change_commit::contracts::KnowledgeChange,
        grounding::{self, ResolvedRunCitation},
        relations, semantic_nodes,
    },
    nodes::contracts::InsertNode,
};

#[derive(Default)]
pub(super) struct ApplyState {
    pub(super) node_id_by_temp_id: HashMap<String, String>,
    pub(super) relation_id_by_temp_id: HashMap<String, String>,
    pub(super) resolved_id_by_temp_id: HashMap<String, String>,
    pub(super) citation_by_cite_id: HashMap<String, ResolvedRunCitation>,
    pub(super) created_nodes: Vec<InsertNode>,
}

/// Resolves every `AddCitedProvenance` citation upfront before creating objects or layouts,
/// failing fast on missing runs or invalid `citeId`s and avoiding duplicate lookups.
pub(super) async fn resolve_run_citations(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    run_id: Option<&str>,
    operations: &[KnowledgeChange],
) -> Result<HashMap<String, ResolvedRunCitation>, String> {
    let mut citation_by_cite_id = HashMap::new();
    for operation in operations {
        let KnowledgeChange::AddCitedProvenance { cite_id, .. } = operation else {
            continue;
        };
        if citation_by_cite_id.contains_key(cite_id.as_str()) {
            continue;
        }
        let active_run_id = run_id.ok_or(
            "add-cited-provenance requires an active conversation run with issued citeIds",
        )?;
        let resolved =
            grounding::resolve_run_citation(tx, space_id, active_run_id, cite_id).await?;
        citation_by_cite_id.insert(cite_id.clone(), resolved);
    }
    Ok(citation_by_cite_id)
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
