use std::collections::{HashMap, HashSet};

use super::planner_support::{apply_sequences, effective_roots, require_editable, Index};
use super::repository::SpaceStructureSnapshot;
use super::{StructureLayout, StructureNode};

pub(super) struct Deletion {
    pub root_ids: Vec<String>,
    pub structure: Vec<StructureNode>,
    pub layouts: Vec<StructureLayout>,
    pub node_ids: Vec<String>,
}

pub(super) fn plan(
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
    let mut pending = roots.clone();
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
        pending.extend(index.children.get(&Some(id)).cloned().unwrap_or_default());
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
    let deleted_structure = original_structure
        .iter()
        .filter(|structure_node| deleted.contains(structure_node.id()))
        .cloned()
        .collect();
    let deleted_layouts = original_layouts
        .iter()
        .filter(|layout| deleted.contains(layout.structure_node_id()))
        .cloned()
        .collect();
    snapshot
        .structure
        .retain(|structure_node| !deleted.contains(structure_node.id()));
    snapshot
        .layouts
        .retain(|layout| !deleted.contains(layout.structure_node_id()));
    let sequences: HashMap<Option<String>, Vec<String>> = index
        .children
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
    Ok(Deletion {
        root_ids: roots,
        structure: deleted_structure,
        layouts: deleted_layouts,
        node_ids,
    })
}
