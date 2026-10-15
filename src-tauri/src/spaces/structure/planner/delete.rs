use std::collections::{HashMap, HashSet};

use crate::spaces::structure::planner::tree_policy::{
    apply_sequences, effective_roots, require_editable, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use crate::spaces::structure::StructureNode;

pub(in crate::spaces::structure) struct Deletion {
    pub node_ids: Vec<String>,
}

pub(in crate::spaces::structure) fn plan(
    snapshot: &mut SpaceStructureSnapshot,
    requested: &[String],
) -> Result<Deletion, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    let roots = effective_roots(&original_structure, &index, requested)?;
    if roots.is_empty() {
        return Err("Deletion requires selected structure nodes".into());
    }
    for id in &roots {
        require_editable(&original_structure, &index, id)?;
    }
    let mut deleted = HashSet::new();
    let mut node_ids = Vec::new();
    let mut pending = roots;
    while let Some(id) = pending.pop() {
        if !deleted.insert(id.clone()) {
            continue;
        }
        let structure_node = index.node(&original_structure, &id)?;
        if structure_node.locked() {
            return Err(format!("Protected descendant prevents deletion: {id}"));
        }
        if let StructureNode::Content { node_id, .. } = structure_node {
            node_ids.push(node_id.clone());
        }
        pending.extend(
            index
                .children_by_parent_id
                .get(&Some(id))
                .cloned()
                .unwrap_or_default(),
        );
    }
    loop {
        let empty = original_structure
            .iter()
            .find(|structure_node| {
                structure_node.kind_name() == "group"
                    && !deleted.contains(structure_node.id())
                    && !original_structure.iter().any(|child| {
                        child.parent_id() == Some(structure_node.id())
                            && !deleted.contains(child.id())
                    })
            })
            .map(|structure_node| structure_node.id().to_owned());
        let Some(id) = empty else { break };
        deleted.insert(id);
    }
    snapshot
        .structure
        .retain(|structure_node| !deleted.contains(structure_node.id()));
    snapshot
        .layouts
        .retain(|layout| !deleted.contains(layout.structure_node_id()));
    let sequences: HashMap<Option<String>, Vec<String>> = index
        .children_by_parent_id
        .iter()
        .map(|(parent, ids)| {
            (
                parent.clone(),
                ids.iter()
                    .filter(|id| !deleted.contains(id.as_str()))
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    apply_sequences(&mut snapshot.structure, &sequences)?;
    Ok(Deletion { node_ids })
}
