use crate::spaces::structure::{StructureLayout, StructureNode};
use std::collections::HashMap;

pub(super) fn remap_structure_node(
    structure_node: StructureNode,
    ids: &HashMap<String, String>,
    nodes: &HashMap<String, String>,
) -> StructureNode {
    let id = ids[structure_node.id()].clone();
    let parent_id = structure_node.parent_id().map(|parent| ids[parent].clone());
    let order = structure_node.order();
    let locked = structure_node.locked();
    match structure_node {
        StructureNode::Content { node_id, .. } => StructureNode::Content {
            id,
            parent_id,
            order,
            locked,
            node_id: nodes[&node_id].clone(),
        },
        StructureNode::Group { title, .. } => StructureNode::Group {
            id,
            parent_id,
            order,
            locked,
            title,
        },
        StructureNode::Section { title, .. } => StructureNode::Section {
            id,
            parent_id,
            order,
            locked,
            title,
        },
    }
}

pub(super) fn remap_layout(
    layout: StructureLayout,
    ids: &HashMap<String, String>,
) -> StructureLayout {
    let structure_node_id = ids[layout.structure_node_id()].clone();
    match layout {
        StructureLayout::Content {
            transform,
            size,
            sizing_mode,
            appearance,
            file_subpath,
            ..
        } => StructureLayout::Content {
            structure_node_id,
            transform,
            size,
            sizing_mode,
            appearance,
            file_subpath,
        },
        StructureLayout::Group { transform, .. } => StructureLayout::Group {
            structure_node_id,
            transform,
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
