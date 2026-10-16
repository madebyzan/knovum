use super::{
    planner::bounds::{bounds_for_roots, merge, root_bounds, Bounds},
    planner::tree_policy::Index,
    StructureLayout, StructureNode,
};

/// Clearance kept between canvas content, both around a placement and between siblings placed as
/// one group.
pub(crate) const CONTENT_GAP: f64 = 48.0;

pub(crate) fn place_new_content(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    source_node_ids: &[String],
    width: f64,
    height: f64,
) -> Result<(f64, f64), String> {
    let index = Index::new(structure, layouts);
    let sources = source_node_ids
        .iter()
        .map(|node_id| {
            structure
                .iter()
                .find_map(|node| match node {
                    StructureNode::Content {
                        id,
                        node_id: resource_id,
                        ..
                    } if resource_id == node_id => Some(id.clone()),
                    _ => None,
                })
                .ok_or_else(|| format!("canvas-placement-invalid: source {node_id} has no layout"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let occupied = structure
        .iter()
        .filter(|node| node.parent_id().is_none())
        .map(|node| root_bounds(structure, layouts, &index, node.id()))
        .collect::<Result<Vec<_>, _>>()?;
    let anchor = if sources.is_empty() {
        let Some(bounds) = occupied
            .iter()
            .fold(None, |all, item| Some(merge(all, *item)))
        else {
            return Ok((0.0, 0.0));
        };
        bounds
    } else {
        bounds_for_roots(structure, layouts, &index, &sources)?
            .ok_or("canvas-placement-invalid: source has no visible bounds")?
    };
    for column in 0..32 {
        for row in 0..32 {
            let offset = if row == 0 {
                0
            } else if row % 2 == 1 {
                (row + 1) / 2
            } else {
                -(row / 2)
            };
            let x = anchor.right + CONTENT_GAP + f64::from(column) * (width + CONTENT_GAP);
            let y = anchor.top + f64::from(offset) * (height + CONTENT_GAP);
            let candidate = Bounds {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            };
            if candidate.left.abs() > 1_000_000_000.0 || candidate.top.abs() > 1_000_000_000.0 {
                continue;
            }
            if occupied
                .iter()
                .all(|existing| !intersects(&candidate, existing))
            {
                return Ok((x, y));
            }
        }
    }
    Err("canvas-placement-unavailable: no clear area beside the sources".into())
}

fn intersects(left: &Bounds, right: &Bounds) -> bool {
    left.left < right.right + CONTENT_GAP
        && left.right + CONTENT_GAP > right.left
        && left.top < right.bottom + CONTENT_GAP
        && left.bottom + CONTENT_GAP > right.top
}

#[cfg(test)]
mod tests;
