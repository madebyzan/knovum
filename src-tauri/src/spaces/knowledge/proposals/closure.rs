//! Which operations of a Change Set need which others.
//!
//! An operation that names another operation's `temp_id` cannot be committed without it: a
//! relation to a claim the person deselected would point at nothing. Acceptance of a subset keeps
//! only operations whose dependencies are kept too, so a partially accepted proposal is always a
//! valid Change Set.

use std::collections::{HashMap, HashSet};

use crate::spaces::knowledge::change_commit::KnowledgeChange;

/// For every operation, the indexes of the operations it references by temp id.
pub(crate) fn dependencies(changes: &[KnowledgeChange]) -> Vec<Vec<usize>> {
    let by_temp_id: HashMap<&str, usize> = changes
        .iter()
        .enumerate()
        .filter_map(|(index, change)| temp_id(change).map(|id| (id, index)))
        .collect();
    changes
        .iter()
        .map(|change| {
            let mut required: Vec<usize> = references(change)
                .into_iter()
                .filter_map(|id| by_temp_id.get(id).copied())
                .collect();
            required.sort_unstable();
            required.dedup();
            required
        })
        .collect()
}

/// The largest part of `selected` whose operations have all their dependencies inside it.
pub(crate) fn close(changes: &[KnowledgeChange], selected: &[usize]) -> Vec<usize> {
    let required = dependencies(changes);
    let mut kept: HashSet<usize> = selected
        .iter()
        .copied()
        .filter(|index| *index < changes.len())
        .collect();
    loop {
        let unmet: Vec<usize> = kept
            .iter()
            .copied()
            .filter(|index| required[*index].iter().any(|needed| !kept.contains(needed)))
            .collect();
        if unmet.is_empty() {
            break;
        }
        for index in unmet {
            kept.remove(&index);
        }
    }
    let mut kept: Vec<usize> = kept.into_iter().collect();
    kept.sort_unstable();
    kept
}

pub(crate) fn temp_id(change: &KnowledgeChange) -> Option<&str> {
    match change {
        KnowledgeChange::CreateObject { temp_id, .. }
        | KnowledgeChange::CreateRelation { temp_id, .. } => Some(temp_id),
        KnowledgeChange::UpdateObject { .. } | KnowledgeChange::AddProvenance { .. } => None,
    }
}

/// Every identity an operation names: an existing node or relation, or another operation's temp id.
pub(crate) fn references(change: &KnowledgeChange) -> Vec<&str> {
    match change {
        KnowledgeChange::CreateObject { .. } => Vec::new(),
        KnowledgeChange::UpdateObject { node_id, .. } => vec![node_id],
        KnowledgeChange::CreateRelation {
            source_id,
            target_id,
            ..
        } => vec![source_id, target_id],
        KnowledgeChange::AddProvenance {
            target_id,
            source_node_id,
            ..
        } => vec![target_id, source_node_id],
    }
}

#[cfg(test)]
#[path = "tests/closure.rs"]
mod tests;
