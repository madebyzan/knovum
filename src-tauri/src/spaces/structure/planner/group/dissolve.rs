use std::collections::{HashMap, HashSet};

use crate::spaces::structure::planner::spatial::layout_keeping_world;
use crate::spaces::structure::planner::tree_policy::{
    apply_sequences, effective_roots, require_editable, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;

pub(in crate::spaces::structure) fn dissolve(
    snapshot: &mut SpaceStructureSnapshot,
    requested: &[String],
) -> Result<Vec<String>, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    let groups = effective_roots(&original_structure, &index, requested)?;
    if groups.is_empty()
        || groups
            .iter()
            .any(|id| index.node(&original_structure, id).unwrap().kind_name() != "group")
    {
        return Err("Dissolve requires selected Group roots".into());
    }
    for id in &groups {
        require_editable(&original_structure, &index, id)?;
    }
    let removed: HashSet<&str> = groups.iter().map(String::as_str).collect();
    let mut promoted = Vec::new();
    let mut updates = HashMap::new();
    for group_id in &groups {
        let parent = index.node(&original_structure, group_id)?.parent_id();
        for child in index
            .children_by_parent_id
            .get(&Some(group_id.clone()))
            .cloned()
            .unwrap_or_default()
        {
            promoted.push(child.clone());
            updates.insert(
                child.clone(),
                layout_keeping_world(
                    &original_structure,
                    &original_layouts,
                    &index,
                    &child,
                    parent,
                )?,
            );
        }
    }
    let mut sequences = HashMap::new();
    for parent in groups
        .iter()
        .map(|id| {
            index
                .node(&original_structure, id)
                .unwrap()
                .parent_id()
                .map(str::to_owned)
        })
        .collect::<HashSet<_>>()
    {
        let ids = index
            .children_by_parent_id
            .get(&parent)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .flat_map(|id| {
                if removed.contains(id.as_str()) {
                    index
                        .children_by_parent_id
                        .get(&Some(id))
                        .cloned()
                        .unwrap_or_default()
                } else {
                    vec![id]
                }
            })
            .collect();
        sequences.insert(parent, ids);
    }
    snapshot
        .structure
        .retain(|node| !removed.contains(node.id()));
    snapshot
        .layouts
        .retain(|layout| !removed.contains(layout.structure_node_id()));
    for layout in &mut snapshot.layouts {
        if let Some(update) = updates.get(layout.structure_node_id()) {
            *layout = update.clone();
        }
    }
    apply_sequences(&mut snapshot.structure, &sequences)?;
    Ok(promoted)
}
