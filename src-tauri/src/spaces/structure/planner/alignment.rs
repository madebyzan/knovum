use std::collections::HashMap;

use crate::spaces::structure::edit_contract::{AlignmentEdge, DistributionAxis};
use crate::spaces::structure::planner::bounds::{merge, root_bounds, Bounds};
use crate::spaces::structure::planner::movement::{moved_layout, replace_layouts};
use crate::spaces::structure::planner::section::require_freely_arranged;
use crate::spaces::structure::planner::tree_policy::{effective_roots, require_editable, Index};
use crate::spaces::structure::repository::SpaceStructureSnapshot;

pub(in crate::spaces::structure) fn arrange(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_ids: &[String],
    align: Option<&AlignmentEdge>,
    axis: Option<&DistributionAxis>,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let roots = effective_roots(&snapshot.structure, &index, structure_node_ids)?;
    let minimum = if align.is_some() { 2 } else { 3 };
    if roots.len() < minimum {
        return Err("Insufficient operation roots for arrangement".into());
    }
    require_freely_arranged(snapshot, &index, &roots)?;
    let mut measures = roots
        .iter()
        .map(|id| {
            Ok((
                id.clone(),
                root_bounds(&snapshot.structure, &snapshot.layouts, &index, id)?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let area = measures
        .iter()
        .fold(None, |area, (_, bounds)| Some(merge(area, *bounds)))
        .expect("roots.len() >= minimum (checked above) guarantees measures is non-empty");
    let mut deltas = HashMap::new();
    if let Some(edge) = align {
        for (id, b) in &measures {
            deltas.insert(id.clone(), alignment_delta(edge, area, *b));
        }
    } else {
        let horizontal = matches!(axis, Some(DistributionAxis::Horizontal));
        measures.sort_by(|a, b| {
            if horizontal {
                a.1.left.total_cmp(&b.1.left)
            } else {
                a.1.top.total_cmp(&b.1.top)
            }
        });
        let occupied: f64 = measures
            .iter()
            .map(|(_, b)| {
                if horizontal {
                    b.right - b.left
                } else {
                    b.bottom - b.top
                }
            })
            .sum();
        let span = if horizontal {
            area.right - area.left
        } else {
            area.bottom - area.top
        };
        let gap = (span - occupied) / (measures.len() - 1) as f64;
        let mut cursor = if horizontal { area.left } else { area.top };
        for (id, b) in &measures {
            let delta = cursor - if horizontal { b.left } else { b.top };
            deltas.insert(
                id.clone(),
                if horizontal {
                    (delta, 0.0)
                } else {
                    (0.0, delta)
                },
            );
            cursor += (if horizontal {
                b.right - b.left
            } else {
                b.bottom - b.top
            }) + gap;
        }
    }
    let mut updates = HashMap::new();
    for id in &roots {
        require_editable(&snapshot.structure, &index, id)?;
        let (dx, dy) = deltas[id];
        updates.insert(id.clone(), moved_layout(snapshot, &index, id, dx, dy)?);
    }
    replace_layouts(&mut snapshot.layouts, updates);
    Ok(roots)
}

fn alignment_delta(edge: &AlignmentEdge, a: Bounds, b: Bounds) -> (f64, f64) {
    match edge {
        AlignmentEdge::Left => (a.left - b.left, 0.0),
        AlignmentEdge::Right => (a.right - b.right, 0.0),
        AlignmentEdge::Center => ((a.left + a.right - b.left - b.right) / 2.0, 0.0),
        AlignmentEdge::Top => (0.0, a.top - b.top),
        AlignmentEdge::Bottom => (0.0, a.bottom - b.bottom),
        AlignmentEdge::Middle => (0.0, (a.top + a.bottom - b.top - b.bottom) / 2.0),
    }
}
