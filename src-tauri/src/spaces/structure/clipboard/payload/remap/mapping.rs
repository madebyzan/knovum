use crate::spaces::structure::{StructureLayout, StructureNode};
use std::collections::HashMap;

pub(super) fn remap_structure_node(
    structure_node: StructureNode,
    structure_node_id_by_source_id: &HashMap<String, String>,
    node_id_by_source_id: &HashMap<String, String>,
) -> StructureNode {
    let id = structure_node_id_by_source_id[structure_node.id()].clone();
    let parent_id = structure_node
        .parent_id()
        .map(|parent| structure_node_id_by_source_id[parent].clone());
    let order = structure_node.order();
    let locked = structure_node.locked();
    match structure_node {
        StructureNode::Content { node_id, .. } => StructureNode::Content {
            id,
            parent_id,
            order,
            locked,
            node_id: node_id_by_source_id[&node_id].clone(),
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
    structure_node_id_by_source_id: &HashMap<String, String>,
) -> StructureLayout {
    let structure_node_id = structure_node_id_by_source_id[layout.structure_node_id()].clone();
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
