use super::super::edit_contract::SpacePoint;
use super::super::planner_spatial::{local_from_world, world_transform};
use super::super::planner_support::Index;
use super::super::{LocalTransform, StructureLayout, StructureNode};
use std::collections::HashMap;

pub(super) fn subtree_ids(index: &Index, roots: &[String]) -> Vec<String> {
    fn append(index: &Index, id: &str, output: &mut Vec<String>) {
        output.push(id.to_owned());
        for child in index
            .children
            .get(&Some(id.to_owned()))
            .into_iter()
            .flatten()
        {
            append(index, child, output);
        }
    }
    let mut result = Vec::new();
    for root in roots {
        append(index, root, &mut result);
    }
    result
}

pub(super) fn clone_structure_node(
    structure_node: &StructureNode,
    id: String,
    parent_id: Option<String>,
    order: u32,
    node_ids: &HashMap<String, String>,
) -> StructureNode {
    match structure_node {
        StructureNode::Content {
            locked, node_id, ..
        } => StructureNode::Content {
            id,
            parent_id,
            order,
            locked: *locked,
            node_id: node_ids[node_id].clone(),
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

pub(super) fn clone_layout(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    index: &Index,
    old_id: &str,
    layout: StructureLayout,
    structure_node_id: String,
    is_root: bool,
    target_world: &LocalTransform,
    offset: &SpacePoint,
) -> Result<StructureLayout, String> {
    if let StructureLayout::Section {
        mut position,
        size,
        collapsed,
        appearance,
        ..
    } = layout
    {
        position.x += offset.x;
        position.y += offset.y;
        return Ok(StructureLayout::Section {
            structure_node_id,
            position,
            size,
            collapsed,
            appearance,
        });
    }
    let transform = if is_root {
        let mut world = world_transform(structure, layouts, index, old_id)?;
        world.x += offset.x;
        world.y += offset.y;
        local_from_world(&world, target_world)
    } else {
        match &layout {
            StructureLayout::Content { transform, .. }
            | StructureLayout::Group { transform, .. } => transform.clone(),
            StructureLayout::Section { .. } => unreachable!(),
        }
    };
    Ok(match layout {
        StructureLayout::Content {
            size,
            size_authority,
            appearance,
            file_subpath,
            ..
        } => StructureLayout::Content {
            structure_node_id,
            transform,
            size,
            size_authority,
            appearance,
            file_subpath,
        },
        StructureLayout::Group { .. } => StructureLayout::Group {
            structure_node_id,
            transform,
        },
        StructureLayout::Section { .. } => unreachable!(),
    })
}
