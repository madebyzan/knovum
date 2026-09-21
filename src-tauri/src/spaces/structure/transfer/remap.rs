use super::super::planner_support::Index;
use super::super::{LocalTransform, StructureLayout, StructureNode};
use std::collections::HashMap;

pub(super) fn subtree_ids(index: &Index, roots: &[String]) -> Vec<String> {
    fn append(index: &Index, id: &str, out: &mut Vec<String>) {
        out.push(id.to_owned());
        for child in index
            .children
            .get(&Some(id.to_owned()))
            .into_iter()
            .flatten()
        {
            append(index, child, out);
        }
    }
    let mut out = Vec::new();
    for root in roots {
        append(index, root, &mut out);
    }
    out
}
pub(super) fn remap_structure_node(
    structure_node: &StructureNode,
    id: String,
    parent_id: Option<String>,
    order: u32,
) -> StructureNode {
    match structure_node {
        StructureNode::Content {
            locked, node_id, ..
        } => StructureNode::Content {
            id,
            parent_id,
            order,
            locked: *locked,
            node_id: node_id.clone(),
        },
        StructureNode::Group { locked, title, .. } => StructureNode::Group {
            id,
            parent_id,
            order,
            locked: *locked,
            title: title.clone(),
        },
        StructureNode::Section { locked, title, .. } => StructureNode::Section {
            id,
            parent_id,
            order,
            locked: *locked,
            title: title.clone(),
        },
    }
}
pub(super) fn remap_layout(
    layout: StructureLayout,
    structure_node_id: String,
    root_transform: Option<LocalTransform>,
) -> StructureLayout {
    match layout {
        StructureLayout::Content {
            transform,
            size,
            size_authority,
            appearance,
            file_subpath,
            ..
        } => StructureLayout::Content {
            structure_node_id,
            transform: root_transform.unwrap_or(transform),
            size,
            size_authority,
            appearance,
            file_subpath,
        },
        StructureLayout::Group { transform, .. } => StructureLayout::Group {
            structure_node_id,
            transform: root_transform.unwrap_or(transform),
        },
        StructureLayout::Section {
            position,
            size,
            collapsed,
            appearance,
            ..
        } => StructureLayout::Section {
            structure_node_id,
            position,
            size,
            collapsed,
            appearance,
        },
    }
}
fn set_order(structure_node: &mut StructureNode, order: u32) {
    match structure_node {
        StructureNode::Content { order: value, .. }
        | StructureNode::Group { order: value, .. }
        | StructureNode::Section { order: value, .. } => *value = order,
    }
}
pub(super) fn normalize_orders(structure: &mut [StructureNode]) {
    let mut groups: HashMap<Option<String>, Vec<(usize, u32)>> = HashMap::new();
    for (index, structure_node) in structure.iter().enumerate() {
        groups
            .entry(structure_node.parent_id().map(str::to_owned))
            .or_default()
            .push((index, structure_node.order()));
    }
    for siblings in groups.values_mut() {
        siblings.sort_by_key(|(_, order)| *order);
        for (order, (index, _)) in siblings.iter().enumerate() {
            set_order(&mut structure[*index], order as u32);
        }
    }
}
pub(super) fn depth(structure: &[StructureNode], index: &Index, id: &str) -> usize {
    let mut depth = 0;
    let mut parent = index
        .node(structure, id)
        .ok()
        .and_then(StructureNode::parent_id);
    while let Some(value) = parent {
        depth += 1;
        parent = index
            .node(structure, value)
            .ok()
            .and_then(StructureNode::parent_id);
    }
    depth
}
