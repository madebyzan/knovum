use super::{
    planner::bounds::{bounds_for_roots, merge, root_bounds, Bounds},
    planner::tree_policy::Index,
    StructureLayout, StructureNode,
};

const GAP: f64 = 48.0;

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
            let x = anchor.right + GAP + f64::from(column) * (width + GAP);
            let y = anchor.top + f64::from(offset) * (height + GAP);
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
    left.left < right.right + GAP
        && left.right + GAP > right.left
        && left.top < right.bottom + GAP
        && left.bottom + GAP > right.top
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spaces::structure::{Point, Size};

    #[test]
    fn unanchored_content_is_placed_beside_sections() {
        let structure = vec![StructureNode::Section {
            id: "section".into(),
            parent_id: None,
            order: 0,
            locked: false,
            title: "Section".into(),
        }];
        let layouts = vec![StructureLayout::Section {
            structure_node_id: "section".into(),
            position: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 400.0,
                height: 300.0,
            },
            collapsed: false,
            appearance: None,
        }];
        assert_eq!(
            place_new_content(&structure, &layouts, &[], 280.0, 350.0).unwrap(),
            (448.0, 0.0)
        );
        assert_eq!(
            place_new_content(&[], &[], &[], 280.0, 350.0).unwrap(),
            (0.0, 0.0)
        );
    }
}
