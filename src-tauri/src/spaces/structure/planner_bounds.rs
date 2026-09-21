use super::contract::{StructureLayout, StructureNode};
use super::planner_spatial::world_transform;
use super::planner_support::Index;

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

pub(super) fn bounds_for_roots(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    index: &Index,
    roots: &[String],
) -> Result<Option<Bounds>, String> {
    let mut pending = roots.to_vec();
    let mut bounds = None;
    while let Some(id) = pending.pop() {
        let structure_node = index.node(structure, &id)?;
        if let StructureNode::Content { .. } = structure_node {
            let StructureLayout::Content { size, .. } = &layouts[index.layout[&id]] else {
                return Err(format!("Content layout missing: {id}"));
            };
            let world = world_transform(structure, layouts, index, &id)?;
            let radians = world.rotation.to_radians();
            for (x, y) in [
                (0.0, 0.0),
                (size.width, 0.0),
                (0.0, size.height),
                (size.width, size.height),
            ] {
                let px = world.x + world.scale * (x * radians.cos() - y * radians.sin());
                let py = world.y + world.scale * (x * radians.sin() + y * radians.cos());
                validate_point(px, py, &id)?;
                bounds = Some(merge(
                    bounds,
                    Bounds {
                        left: px,
                        top: py,
                        right: px,
                        bottom: py,
                    },
                ));
            }
        } else {
            pending.extend(index.children.get(&Some(id)).cloned().unwrap_or_default());
        }
    }
    Ok(bounds)
}

pub(super) fn root_bounds(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    index: &Index,
    id: &str,
) -> Result<Bounds, String> {
    if index.node(structure, id)?.kind_name() == "section" {
        let StructureLayout::Section { position, size, .. } = &layouts[index.layout[id]] else {
            return Err(format!("Section layout missing: {id}"));
        };
        return Ok(Bounds {
            left: position.x,
            top: position.y,
            right: position.x + size.width,
            bottom: position.y + size.height,
        });
    }
    bounds_for_roots(structure, layouts, index, &[id.to_owned()])?
        .ok_or_else(|| format!("Operation root has no geometry: {id}"))
}

pub(super) fn merge(left: Option<Bounds>, right: Bounds) -> Bounds {
    match left {
        None => right,
        Some(left) => Bounds {
            left: left.left.min(right.left),
            top: left.top.min(right.top),
            right: left.right.max(right.right),
            bottom: left.bottom.max(right.bottom),
        },
    }
}

fn validate_point(x: f64, y: f64, structure_node_id: &str) -> Result<(), String> {
    if x.is_finite() && y.is_finite() && x.abs() <= 1_000_000_000.0 && y.abs() <= 1_000_000_000.0 {
        Ok(())
    } else {
        Err(format!(
            "Content geometry exceeds bounds: {structure_node_id}"
        ))
    }
}
