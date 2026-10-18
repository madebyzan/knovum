use super::{
    geometry::MAX_COORDINATE,
    planner::bounds::{bounds_for_roots, merge, root_bounds, Bounds},
    planner::tree_policy::Index,
    StructureLayout, StructureNode,
};

/// Clearance kept between canvas content, both around a placement and between siblings placed as
/// one group.
pub(crate) const CONTENT_GAP: f64 = 48.0;

/// How far a field is followed, in rows and in columns, before the canvas reports that the field
/// beside the sources has no room left.
const FIELD_LIMIT: u32 = 32;

/// A block of derived content looking for its place in the field beside its sources.
pub(crate) struct DerivedBlock<'a> {
    pub(crate) source_node_ids: &'a [String],
    /// Content that belongs to the block itself: it was reserved before the block could be sized,
    /// so it must not block its own placement.
    pub(crate) placed_node_ids: &'a [String],
    /// Derived outputs the sources already carry, which decide how wide the field is.
    pub(crate) siblings: usize,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

/// Places content that has no source to sit beside, in the first clear area past everything the
/// canvas already holds.
pub(crate) fn place_new_content(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    width: f64,
    height: f64,
) -> Result<(f64, f64), String> {
    place_derived_content(
        structure,
        layouts,
        &DerivedBlock {
            source_node_ids: &[],
            placed_node_ids: &[],
            siblings: 0,
            width,
            height,
        },
    )
}

/// Places a block of derived content into its sources' field.
///
/// Everything one source produces accumulates in a single field: a lattice that begins at the
/// right edge of the sources, top-aligned with them, filled in reading order, and kept as close to
/// square as the number of outputs allows. Occupied cells are skipped instead of reclaimed, so
/// nothing already on the canvas moves - including an output someone dragged elsewhere.
pub(crate) fn place_derived_content(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
    block: &DerivedBlock<'_>,
) -> Result<(f64, f64), String> {
    let index = Index::new(structure, layouts);
    let sources = block
        .source_node_ids
        .iter()
        .map(|node_id| {
            structure_id(structure, node_id)
                .ok_or_else(|| format!("canvas-placement-invalid: source {node_id} has no layout"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let placed = block
        .placed_node_ids
        .iter()
        .filter_map(|node_id| structure_id(structure, node_id))
        .collect::<Vec<_>>();
    let occupied = structure
        .iter()
        .filter(|node| {
            node.parent_id().is_none() && !placed.iter().any(|id| id.as_str() == node.id())
        })
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
    let columns = field_columns(block.siblings);
    for row in 0..FIELD_LIMIT {
        for column in 0..columns {
            let x = anchor.right + CONTENT_GAP + f64::from(column) * (block.width + CONTENT_GAP);
            let y = anchor.top + f64::from(row) * (block.height + CONTENT_GAP);
            let candidate = Bounds {
                left: x,
                top: y,
                right: x + block.width,
                bottom: y + block.height,
            };
            if candidate.right.abs() > MAX_COORDINATE || candidate.bottom.abs() > MAX_COORDINATE {
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
    Err("canvas-placement-unavailable: the field beside the sources is full".into())
}

/// A field holds as many columns as keep it close to square, which is the shape rule a set of
/// outputs published together already follows.
fn field_columns(siblings: usize) -> u32 {
    let count = u32::try_from(siblings.saturating_add(1)).unwrap_or(FIELD_LIMIT);
    (1..=FIELD_LIMIT)
        .find(|columns| columns * columns >= count)
        .unwrap_or(FIELD_LIMIT)
}

fn structure_id(structure: &[StructureNode], node_id: &str) -> Option<String> {
    structure.iter().find_map(|node| match node {
        StructureNode::Content {
            id,
            node_id: resource_id,
            ..
        } if resource_id == node_id => Some(id.clone()),
        _ => None,
    })
}

fn intersects(left: &Bounds, right: &Bounds) -> bool {
    left.left < right.right + CONTENT_GAP
        && left.right + CONTENT_GAP > right.left
        && left.top < right.bottom + CONTENT_GAP
        && left.bottom + CONTENT_GAP > right.top
}

#[cfg(test)]
mod tests;
