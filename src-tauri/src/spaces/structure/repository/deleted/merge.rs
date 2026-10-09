use super::super::SpaceStructureSnapshot;
use super::RestoredStructure;
use crate::spaces::structure::StructureNode;
use std::collections::{HashMap, HashSet};

pub(crate) fn merge_restored_snapshot(
    snapshot: &mut SpaceStructureSnapshot,
    restored: &RestoredStructure,
) -> Result<(), String> {
    let mut children_by_parent_id: HashMap<Option<String>, Vec<&StructureNode>> = HashMap::new();
    for structure_node in &restored.structure {
        children_by_parent_id
            .entry(structure_node.parent_id().map(str::to_owned))
            .or_default()
            .push(structure_node);
    }
    snapshot.structure.extend(restored.structure.clone());
    snapshot.layouts.extend(restored.layouts.clone());
    snapshot
        .relation_styles
        .extend(restored.relation_styles.clone());
    for (parent, mut restoring) in children_by_parent_id {
        restoring.sort_by_key(|structure_node| structure_node.order());
        let restoring_ids: HashSet<&str> = restoring
            .iter()
            .map(|structure_node| structure_node.id())
            .collect();
        let mut sequence = snapshot
            .structure
            .iter()
            .filter(|structure_node| {
                structure_node.parent_id() == parent.as_deref()
                    && !restoring_ids.contains(structure_node.id())
            })
            .map(|structure_node| (structure_node.order(), structure_node.id().to_owned()))
            .collect::<Vec<_>>();
        sequence.sort_by_key(|(order, _)| *order);
        let mut ids = sequence.into_iter().map(|(_, id)| id).collect::<Vec<_>>();
        for structure_node in restoring {
            let index = (structure_node.order() as usize).min(ids.len());
            ids.insert(index, structure_node.id().to_owned());
        }
        for (order, id) in ids.into_iter().enumerate() {
            let structure_node = snapshot
                .structure
                .iter_mut()
                .find(|structure_node| structure_node.id() == id)
                .ok_or("Restored structure_node missing")?;
            set_order(
                structure_node,
                u32::try_from(order).map_err(|_| "Restore order overflow")?,
            );
        }
    }
    Ok(())
}

fn set_order(structure_node: &mut StructureNode, order: u32) {
    match structure_node {
        StructureNode::Content { order: value, .. }
        | StructureNode::Group { order: value, .. }
        | StructureNode::Section { order: value, .. } => *value = order,
    }
}
