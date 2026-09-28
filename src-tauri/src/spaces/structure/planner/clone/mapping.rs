use crate::spaces::structure::edit_contract::SpacePoint;
use crate::spaces::structure::planner::spatial::{local_from_world, world_transform};
use crate::spaces::structure::planner::tree_policy::Index;
use crate::spaces::structure::{LocalTransform, StructureLayout, StructureNode};
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
    let mut subtree_ids = Vec::new();
    for root in roots {
        append(index, root, &mut subtree_ids);
    }
    subtree_ids
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

pub(super) struct LayoutContext<'a> {
    pub structure: &'a [StructureNode],
    pub layouts: &'a [StructureLayout],
    pub index: &'a Index,
    pub target_world: &'a LocalTransform,
    pub offset: &'a SpacePoint,
}

pub(super) fn clone_layout(
    context: &LayoutContext<'_>,
    old_id: &str,
    layout: StructureLayout,
    structure_node_id: String,
    is_root: bool,
) -> Result<StructureLayout, String> {
    if let StructureLayout::Section {
        mut position,
        size,
        collapsed,
        appearance,
        ..
    } = layout
    {
        position.x += context.offset.x;
        position.y += context.offset.y;
        return Ok(StructureLayout::Section {
            structure_node_id,
            position,
            size,
            collapsed,
            appearance,
        });
    }
    let transform = if is_root {
        let mut world = world_transform(context.structure, context.layouts, context.index, old_id)?;
        world.x += context.offset.x;
        world.y += context.offset.y;
        local_from_world(&world, context.target_world)
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
        StructureLayout::Group { .. } => StructureLayout::Group {
            structure_node_id,
            transform,
        },
        StructureLayout::Section { .. } => unreachable!(),
    })
}
