use std::collections::{HashMap, HashSet};

use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::spaces::structure::contract::{LocalTransform, StructureNode};
use crate::spaces::structure::edit_contract::SpacePoint;
use crate::spaces::structure::planner::spatial::world_transform;
use crate::spaces::structure::planner::tree_policy::{effective_roots, Index};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
mod mapping;
mod persistence;

pub(in crate::spaces::structure) async fn clone_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &mut SpaceStructureSnapshot,
    requested: &[String],
    target_parent_id: Option<&str>,
    offset: &SpacePoint,
) -> Result<Vec<String>, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    let roots = effective_roots(&original_structure, &index, requested)?;
    if roots.is_empty() {
        return Err("Clone requires selected structure nodes".into());
    }
    if let Some(parent_id) = target_parent_id {
        let parent = index.node(&original_structure, parent_id)?;
        if parent.kind_name() == "content" {
            return Err("Invalid clone target".into());
        }
        if roots.iter().any(|id| {
            index
                .node(&original_structure, id)
                .is_ok_and(|structure_node| structure_node.kind_name() == "section")
        }) {
            return Err("A cloned Section must remain at the Space root".into());
        }
    }

    let moved_ids = mapping::subtree_ids(&index, &roots);
    let mut structure_node_ids = HashMap::new();
    let mut node_ids = HashMap::new();
    for id in &moved_ids {
        structure_node_ids.insert(id.clone(), Uuid::new_v4().to_string());
        if let StructureNode::Content { node_id, .. } = index.node(&original_structure, id)? {
            node_ids.insert(node_id.clone(), Uuid::new_v4().to_string());
        }
    }
    persistence::clone_nodes(tx, space_id, &node_ids, &structure_node_ids).await?;
    persistence::clone_relations(tx, space_id, &node_ids, snapshot).await?;

    let root_set: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let target_key = target_parent_id.map(str::to_owned);
    let mut next_order = snapshot
        .structure
        .iter()
        .filter(|structure_node| structure_node.parent_id() == target_parent_id)
        .map(StructureNode::order)
        .max()
        .map_or(0, |value| value + 1);
    let target_world = match target_parent_id {
        Some(parent) => world_transform(&original_structure, &original_layouts, &index, parent)?,
        None => LocalTransform {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale: 1.0,
        },
    };
    let mut cloned_structure = Vec::with_capacity(moved_ids.len());
    let mut cloned_layouts = Vec::with_capacity(moved_ids.len());
    for old_id in &moved_ids {
        let source_node = index.node(&original_structure, old_id)?;
        let new_id = structure_node_ids[old_id].clone();
        let is_root = root_set.contains(old_id.as_str());
        let parent_id = if is_root {
            target_key.clone()
        } else {
            source_node
                .parent_id()
                .map(|parent| structure_node_ids[parent].clone())
        };
        let order = if is_root {
            let value = next_order;
            next_order += 1;
            value
        } else {
            source_node.order()
        };
        cloned_structure.push(mapping::clone_structure_node(
            source_node,
            new_id.clone(),
            parent_id,
            order,
            &node_ids,
        ));
        let old_layout = original_layouts[index.layout[old_id]].clone();
        cloned_layouts.push(mapping::clone_layout(
            &mapping::LayoutContext {
                structure: &original_structure,
                layouts: &original_layouts,
                index: &index,
                target_world: &target_world,
                offset,
            },
            old_id,
            old_layout,
            new_id,
            is_root,
        )?);
    }
    snapshot.structure.extend(cloned_structure);
    snapshot.layouts.extend(cloned_layouts);
    Ok(roots
        .into_iter()
        .map(|id| structure_node_ids[&id].clone())
        .collect())
}
