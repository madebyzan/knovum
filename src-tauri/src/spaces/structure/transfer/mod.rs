use super::planner::spatial::{local_from_world, world_transform};
use super::planner::tree_policy::{
    effective_roots, remove_empty_groups, require_editable, set_order, Index,
};
use super::repository::SpaceStructureSnapshot;
use super::{LocalTransform, StructureNode};
use std::collections::{HashMap, HashSet};

mod remap;

pub(crate) struct TransferPlan {
    pub source_structure_node_ids_deepest_first: Vec<String>,
    pub moved_structure_node_ids: Vec<String>,
    pub moved_node_ids: Vec<String>,
}

pub(crate) fn plan(
    source: &mut SpaceStructureSnapshot,
    target: &mut SpaceStructureSnapshot,
    requested: &[String],
    target_parent: Option<&str>,
    target_order: u32,
) -> Result<TransferPlan, String> {
    let source_structure = source.structure.clone();
    let source_layouts = source.layouts.clone();
    let source_index = Index::new(&source_structure, &source_layouts);
    let roots = effective_roots(&source_structure, &source_index, requested)?;
    if roots.is_empty() {
        return Err("Cross-Space move requires structure roots".into());
    }
    for root in &roots {
        require_editable(&source_structure, &source_index, root)?;
    }
    let target_index = Index::new(&target.structure, &target.layouts);
    if let Some(parent) = target_parent {
        let structure_node = target_index.node(&target.structure, parent)?;
        if structure_node.kind_name() == "content" {
            return Err("Cross-Space target parent cannot be Content".into());
        }
        require_editable(&target.structure, &target_index, parent)?;
    }
    if target_parent.is_some()
        && roots.iter().any(|id| {
            source_index
                .node(&source_structure, id)
                .is_ok_and(|structure_node| structure_node.kind_name() == "section")
        })
    {
        return Err("A Section must enter the target Space root".into());
    }
    let destination_count = target
        .structure
        .iter()
        .filter(|structure_node| structure_node.parent_id() == target_parent)
        .count();
    if target_order as usize > destination_count {
        return Err("Cross-Space target order is invalid".into());
    }
    let moved = remap::subtree_ids(&source_index, &roots);
    let moved_set: HashSet<&str> = moved.iter().map(String::as_str).collect();
    let moved_node_ids = moved
        .iter()
        .filter_map(|id| match source_index.node(&source_structure, id).ok()? {
            StructureNode::Content { node_id, .. } => Some(node_id.clone()),
            _ => None,
        })
        .collect();
    let ids: HashMap<String, String> = moved.iter().map(|id| (id.clone(), id.clone())).collect();
    let root_count = u32::try_from(roots.len()).map_err(|_| "Cross-Space order overflow")?;
    for structure_node in &mut target.structure {
        if structure_node.parent_id() == target_parent && structure_node.order() >= target_order {
            set_order(
                structure_node,
                structure_node
                    .order()
                    .checked_add(root_count)
                    .ok_or("Cross-Space order overflow")?,
            );
        }
    }
    let parent_world = match target_parent {
        Some(parent) => world_transform(&target.structure, &target.layouts, &target_index, parent)?,
        None => LocalTransform {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale: 1.0,
        },
    };
    let root_set: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let mut root_offset = 0u32;
    for old_id in &moved {
        let structure_node = source_index.node(&source_structure, old_id)?;
        let root = root_set.contains(old_id.as_str());
        let new_id = ids[old_id].clone();
        let parent_id = if root {
            target_parent.map(str::to_owned)
        } else {
            structure_node.parent_id().map(|parent| ids[parent].clone())
        };
        let order = if root {
            let value = target_order + root_offset;
            root_offset += 1;
            value
        } else {
            structure_node.order()
        };
        target.structure.push(remap::remap_structure_node(
            structure_node,
            new_id.clone(),
            parent_id,
            order,
        ));
        let layout = source_layouts[source_index.layout_index_by_structure_node_id[old_id]].clone();
        let root_transform = if root && structure_node.kind_name() != "section" {
            Some(local_from_world(
                &world_transform(&source_structure, &source_layouts, &source_index, old_id)?,
                &parent_world,
            ))
        } else {
            None
        };
        target
            .layouts
            .push(remap::remap_layout(layout, new_id, root_transform));
    }
    source
        .structure
        .retain(|structure_node| !moved_set.contains(structure_node.id()));
    source
        .layouts
        .retain(|layout| !moved_set.contains(layout.structure_node_id()));
    remove_empty_groups(&mut source.structure, &mut source.layouts);
    remap::normalize_orders(&mut source.structure);
    let depths: HashMap<&str, usize> = moved
        .iter()
        .map(|id| {
            (
                id.as_str(),
                remap::depth(&source_structure, &source_index, id),
            )
        })
        .collect();
    let mut deepest = moved.clone();
    deepest.sort_by_key(|id| std::cmp::Reverse(depths[id.as_str()]));
    Ok(TransferPlan {
        source_structure_node_ids_deepest_first: deepest,
        moved_structure_node_ids: moved,
        moved_node_ids,
    })
}

pub(crate) fn move_relation_styles(
    source: &mut SpaceStructureSnapshot,
    target: &mut SpaceStructureSnapshot,
    relation_ids: &[String],
) {
    let ids: HashSet<&str> = relation_ids.iter().map(String::as_str).collect();
    let mut moved = Vec::new();
    source.relation_styles.retain(|style| {
        if ids.contains(style.relation_id.as_str()) {
            moved.push(style.clone());
            false
        } else {
            true
        }
    });
    target.relation_styles.extend(moved);
}
