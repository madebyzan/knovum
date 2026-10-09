use std::collections::HashSet;

use crate::spaces::structure::contracts::{Point, Size, StructureLayout, StructureNode};
use crate::spaces::structure::edit_contract::SpaceRect;

mod content;
use crate::spaces::structure::planner::tree_policy::{effective_roots, require_editable, Index};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
pub(in crate::spaces::structure) use content::{
    relation_style, update_content, update_content_heights,
};

pub(in crate::spaces::structure) fn create_empty(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_id: &str,
    title: &str,
    frame: &SpaceRect,
    order: u32,
) -> Result<Vec<String>, String> {
    if snapshot
        .structure
        .iter()
        .any(|structure_node| structure_node.id() == structure_node_id)
    {
        return Err(format!(
            "Structure node already exists: {structure_node_id}"
        ));
    }
    let root_count = snapshot
        .structure
        .iter()
        .filter(|structure_node| structure_node.parent_id().is_none())
        .count();
    if order as usize > root_count {
        return Err("Invalid Section insertion order".into());
    }
    for structure_node in &mut snapshot.structure {
        if structure_node.parent_id().is_none() && structure_node.order() >= order {
            let next = structure_node.order() + 1;
            set_order(structure_node, next);
        }
    }
    snapshot.structure.push(StructureNode::Section {
        id: structure_node_id.to_owned(),
        parent_id: None,
        order,
        locked: false,
        title: title.to_owned(),
    });
    snapshot.layouts.push(StructureLayout::Section {
        structure_node_id: structure_node_id.to_owned(),
        position: Point {
            x: frame.x,
            y: frame.y,
        },
        size: Size {
            width: frame.width,
            height: frame.height,
        },
        collapsed: false,
        appearance: None,
    });
    Ok(vec![structure_node_id.to_owned()])
}

pub(in crate::spaces::structure) fn collapse(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_id: &str,
    collapsed: bool,
) -> Result<Vec<String>, String> {
    if !snapshot.structure.iter().any(|structure_node| {
        structure_node.id() == structure_node_id && structure_node.kind_name() == "section"
    }) {
        return Err(format!("Section not found: {structure_node_id}"));
    }
    match snapshot
        .layouts
        .iter_mut()
        .find(|layout| layout.structure_node_id() == structure_node_id)
    {
        Some(StructureLayout::Section {
            collapsed: value, ..
        }) => *value = collapsed,
        _ => return Err(format!("Section layout not found: {structure_node_id}")),
    };
    Ok(vec![structure_node_id.to_owned()])
}

pub(in crate::spaces::structure) fn set_lock(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_ids: &[String],
    locked: bool,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let roots = effective_roots(&snapshot.structure, &index, structure_node_ids)?;
    if roots.is_empty() {
        return Err("Locking requires selected structure nodes".into());
    }
    for id in &roots {
        for ancestor in crate::spaces::structure::planner::tree_policy::ancestors(
            &snapshot.structure,
            &index,
            id,
        )? {
            if index.node(&snapshot.structure, &ancestor)?.locked() {
                return Err(format!("Locked ancestor: {id}"));
            }
        }
    }
    let selected: HashSet<&str> = roots.iter().map(String::as_str).collect();
    for structure_node in &mut snapshot.structure {
        if selected.contains(structure_node.id()) {
            set_locked(structure_node, locked);
        }
    }
    Ok(roots)
}

pub(in crate::spaces::structure) fn rename(
    snapshot: &mut SpaceStructureSnapshot,
    container_id: &str,
    title: &str,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    if index.node(&snapshot.structure, container_id)?.kind_name() == "content" {
        return Err(format!("Container not found: {container_id}"));
    }
    require_editable(&snapshot.structure, &index, container_id)?;
    for structure_node in &mut snapshot.structure {
        match structure_node {
            StructureNode::Group {
                id: structure_node_id,
                title: value,
                ..
            }
            | StructureNode::Section {
                id: structure_node_id,
                title: value,
                ..
            } if structure_node_id == container_id => *value = title.to_owned(),
            _ => {}
        }
    }
    Ok(vec![container_id.to_owned()])
}

fn set_order(structure_node: &mut StructureNode, order: u32) {
    match structure_node {
        StructureNode::Content { order: value, .. }
        | StructureNode::Group { order: value, .. }
        | StructureNode::Section { order: value, .. } => *value = order,
    }
}
fn set_locked(structure_node: &mut StructureNode, locked: bool) {
    match structure_node {
        StructureNode::Content { locked: value, .. }
        | StructureNode::Group { locked: value, .. }
        | StructureNode::Section { locked: value, .. } => *value = locked,
    }
}
