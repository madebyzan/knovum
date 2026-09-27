use crate::spaces::structure::planner::spatial::{local_from_world, world_transform};
use crate::spaces::structure::planner::tree_policy::{
    require_editable, set_order, set_parent, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use crate::spaces::structure::{LocalTransform, StructureLayout};

use super::payload::RemappedPayload;

pub(super) fn attach_roots(
    imported: &mut RemappedPayload,
    target: &SpaceStructureSnapshot,
    parent_id: Option<&str>,
) -> Result<(), String> {
    let target_index = Index::new(&target.structure, &target.layouts);
    let parent_world = match parent_id {
        Some(parent) => {
            let structure_node = target_index.node(&target.structure, parent)?;
            if structure_node.kind_name() == "content" {
                return Err("Clipboard target parent cannot be Content".into());
            }
            require_editable(&target.structure, &target_index, parent)?;
            world_transform(&target.structure, &target.layouts, &target_index, parent)?
        }
        None => identity(),
    };
    let mut roots = imported
        .structure
        .iter()
        .filter(|structure_node| structure_node.parent_id().is_none())
        .map(|structure_node| {
            (
                structure_node.order(),
                structure_node.id().to_owned(),
                structure_node.kind_name(),
            )
        })
        .collect::<Vec<_>>();
    roots.sort_by_key(|(order, _, _)| *order);
    if parent_id.is_some() && roots.iter().any(|(_, _, kind)| *kind == "section") {
        return Err("A pasted Section must remain at the Space root".into());
    }
    let first_order = target
        .structure
        .iter()
        .filter(|structure_node| structure_node.parent_id() == parent_id)
        .count();
    for (offset, (_, id, _)) in roots.into_iter().enumerate() {
        let structure_node = imported
            .structure
            .iter_mut()
            .find(|structure_node| structure_node.id() == id)
            .ok_or("Clipboard root missing")?;
        set_parent(structure_node, parent_id.map(str::to_owned));
        set_order(
            structure_node,
            u32::try_from(first_order + offset).map_err(|_| "Clipboard order overflow")?,
        );
        if parent_id.is_some() {
            localize_root(&mut imported.layouts, &id, &parent_world)?;
        }
    }
    Ok(())
}

fn localize_root(
    layouts: &mut [StructureLayout],
    id: &str,
    parent_world: &LocalTransform,
) -> Result<(), String> {
    let layout = layouts
        .iter_mut()
        .find(|layout| layout.structure_node_id() == id)
        .ok_or("Clipboard root layout missing")?;
    match layout {
        StructureLayout::Content { transform, .. } | StructureLayout::Group { transform, .. } => {
            *transform = local_from_world(transform, parent_world);
            Ok(())
        }
        StructureLayout::Section { .. } => {
            Err("A pasted Section must remain at the Space root".into())
        }
    }
}

fn identity() -> LocalTransform {
    LocalTransform {
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        scale: 1.0,
    }
}
