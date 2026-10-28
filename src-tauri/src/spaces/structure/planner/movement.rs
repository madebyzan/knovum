use std::collections::HashMap;

use crate::spaces::structure::contracts::{LocalTransform, StructureLayout};
use crate::spaces::structure::edit_contract::SpacePoint;
use crate::spaces::structure::planner::spatial::{local_from_world, world_transform};
use crate::spaces::structure::planner::tree_policy::{effective_roots, require_editable, Index};
use crate::spaces::structure::repository::SpaceStructureSnapshot;

pub(in crate::spaces::structure) fn move_structure(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_ids: &[String],
    delta: &SpacePoint,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let roots = effective_roots(&snapshot.structure, &index, structure_node_ids)?;
    if roots.is_empty() {
        return Err("Move requires selected structure nodes".into());
    }
    let mut updates = HashMap::new();
    for id in &roots {
        require_editable(&snapshot.structure, &index, id)?;
        updates.insert(
            id.clone(),
            moved_layout(snapshot, &index, id, delta.x, delta.y)?,
        );
    }
    replace_layouts(&mut snapshot.layouts, updates);
    Ok(roots)
}

pub(in crate::spaces::structure) fn moved_layout(
    snapshot: &SpaceStructureSnapshot,
    index: &Index,
    target_structure_node_id: &str,
    dx: f64,
    dy: f64,
) -> Result<StructureLayout, String> {
    let structure_node = index.node(&snapshot.structure, target_structure_node_id)?;
    let mut layout =
        snapshot.layouts[index.layout_index_by_structure_node_id[target_structure_node_id]].clone();
    if let StructureLayout::Section { position, .. } = &mut layout {
        position.x += dx;
        position.y += dy;
        return Ok(layout);
    }
    let parent = match structure_node.parent_id() {
        Some(parent) => world_transform(&snapshot.structure, &snapshot.layouts, index, parent)?,
        None => LocalTransform {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale: 1.0,
        },
    };
    let radians = parent.rotation.to_radians();
    map_transform(layout, |mut transform| {
        transform.x += (dx * radians.cos() + dy * radians.sin()) / parent.scale;
        transform.y += (-dx * radians.sin() + dy * radians.cos()) / parent.scale;
        transform
    })
}

pub(in crate::spaces::structure) fn transform_structure(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_ids: &[String],
    anchor: &SpacePoint,
    translation: &SpacePoint,
    rotation_delta: f64,
    scale_factor: f64,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let roots = effective_roots(&snapshot.structure, &index, structure_node_ids)?;
    if roots.is_empty()
        || roots.iter().any(|id| {
            index
                .node(&snapshot.structure, id)
                .expect("effective_roots ids were already validated against the structure index")
                .kind_name()
                == "section"
        })
    {
        return Err("Transform requires selected Content or Group roots".into());
    }
    if scale_factor <= 0.0 {
        return Err("Invalid structure_node transform".into());
    }
    let radians = rotation_delta.to_radians();
    let mut layout_by_structure_node_id = HashMap::new();
    for id in &roots {
        require_editable(&snapshot.structure, &index, id)?;
        let before = world_transform(&snapshot.structure, &snapshot.layouts, &index, id)?;
        let dx = before.x - anchor.x;
        let dy = before.y - anchor.y;
        let after = LocalTransform {
            x: anchor.x + scale_factor * (dx * radians.cos() - dy * radians.sin()),
            y: anchor.y + scale_factor * (dx * radians.sin() + dy * radians.cos()),
            rotation: before.rotation + rotation_delta,
            scale: before.scale * scale_factor,
        };
        let parent = match index.node(&snapshot.structure, id)?.parent_id() {
            Some(parent) => {
                world_transform(&snapshot.structure, &snapshot.layouts, &index, parent)?
            }
            None => LocalTransform {
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
                scale: 1.0,
            },
        };
        layout_by_structure_node_id.insert(
            id.clone(),
            map_transform(
                snapshot.layouts[index.layout_index_by_structure_node_id[id]].clone(),
                |_| local_from_world(&after, &parent),
            )?,
        );
    }
    replace_layouts(&mut snapshot.layouts, layout_by_structure_node_id);
    if translation.x != 0.0 || translation.y != 0.0 {
        move_structure(snapshot, structure_node_ids, translation)?;
    }
    Ok(roots)
}

fn map_transform(
    layout: StructureLayout,
    map: impl FnOnce(LocalTransform) -> LocalTransform,
) -> Result<StructureLayout, String> {
    Ok(match layout {
        StructureLayout::Content {
            structure_node_id,
            transform,
            size,
            sizing_mode,
            appearance,
            file_subpath,
        } => StructureLayout::Content {
            structure_node_id,
            transform: map(transform),
            size,
            sizing_mode,
            appearance,
            file_subpath,
        },
        StructureLayout::Group {
            structure_node_id,
            transform,
        } => StructureLayout::Group {
            structure_node_id,
            transform: map(transform),
        },
        StructureLayout::Section { .. } => return Err("Section has no transform".into()),
    })
}
pub(in crate::spaces::structure) fn replace_layouts(
    layouts: &mut [StructureLayout],
    layout_by_structure_node_id: HashMap<String, StructureLayout>,
) {
    for layout in layouts {
        if let Some(update) = layout_by_structure_node_id.get(layout.structure_node_id()) {
            *layout = update.clone();
        }
    }
}
