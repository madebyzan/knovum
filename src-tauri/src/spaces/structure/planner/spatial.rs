use crate::spaces::structure::contract::{LocalTransform, StructureLayout, StructureNode};
use crate::spaces::structure::planner::tree_policy::{ancestors, Index};

pub(in crate::spaces::structure) fn world_transform(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    index: &Index,
    id: &str,
) -> Result<LocalTransform, String> {
    let mut world = LocalTransform {
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        scale: 1.0,
    };
    for member in ancestors(structure, index, id)?
        .into_iter()
        .chain([id.to_owned()])
    {
        let layout = &layouts[*index
            .layout
            .get(&member)
            .ok_or_else(|| format!("Layout missing: {member}"))?];
        world = compose(&world, &local_transform(layout));
    }
    Ok(world)
}

pub(in crate::spaces::structure) fn local_transform(layout: &StructureLayout) -> LocalTransform {
    match layout {
        StructureLayout::Content { transform, .. } | StructureLayout::Group { transform, .. } => {
            transform.clone()
        }
        StructureLayout::Section { position, .. } => LocalTransform {
            x: position.x,
            y: position.y,
            rotation: 0.0,
            scale: 1.0,
        },
    }
}

pub(in crate::spaces::structure) fn compose(
    parent: &LocalTransform,
    local: &LocalTransform,
) -> LocalTransform {
    let radians = parent.rotation.to_radians();
    LocalTransform {
        x: parent.x + parent.scale * (local.x * radians.cos() - local.y * radians.sin()),
        y: parent.y + parent.scale * (local.x * radians.sin() + local.y * radians.cos()),
        rotation: parent.rotation + local.rotation,
        scale: parent.scale * local.scale,
    }
}

pub(in crate::spaces::structure) fn local_from_world(
    world: &LocalTransform,
    parent: &LocalTransform,
) -> LocalTransform {
    let radians = parent.rotation.to_radians();
    let dx = world.x - parent.x;
    let dy = world.y - parent.y;
    LocalTransform {
        x: (dx * radians.cos() + dy * radians.sin()) / parent.scale,
        y: (-dx * radians.sin() + dy * radians.cos()) / parent.scale,
        rotation: world.rotation - parent.rotation,
        scale: world.scale / parent.scale,
    }
}

pub(in crate::spaces::structure) fn layout_keeping_world(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    index: &Index,
    id: &str,
    new_parent: Option<&str>,
) -> Result<StructureLayout, String> {
    let world = world_transform(structure, layouts, index, id)?;
    let parent = match new_parent {
        Some(parent) => world_transform(structure, layouts, index, parent)?,
        None => LocalTransform {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale: 1.0,
        },
    };
    let local = local_from_world(&world, &parent);
    let layout = layouts[*index
        .layout
        .get(id)
        .ok_or_else(|| format!("Layout missing: {id}"))?]
    .clone();
    Ok(match layout {
        StructureLayout::Content {
            structure_node_id,
            size,
            size_authority,
            appearance,
            file_subpath,
            ..
        } => StructureLayout::Content {
            structure_node_id,
            transform: local,
            size,
            size_authority,
            appearance,
            file_subpath,
        },
        StructureLayout::Group {
            structure_node_id, ..
        } => StructureLayout::Group {
            structure_node_id,
            transform: local,
        },
        section @ StructureLayout::Section { .. } => section,
    })
}
