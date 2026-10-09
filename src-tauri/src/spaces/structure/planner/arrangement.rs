use std::collections::{HashMap, HashSet};

use crate::spaces::structure::edit_contract::{AutoLayoutDirection, ReorderDirection};
use crate::spaces::structure::planner::bounds::{merge, root_bounds};
use crate::spaces::structure::planner::movement::{moved_layout, replace_layouts};
use crate::spaces::structure::planner::tree_policy::{
    apply_sequences, effective_roots, require_editable, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;

pub(in crate::spaces::structure) fn reorder(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_ids: &[String],
    direction: &ReorderDirection,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let roots = effective_roots(&snapshot.structure, &index, structure_node_ids)?;
    if roots.is_empty() {
        return Err("Reorder requires selected roots".into());
    }
    let parent = index
        .node(&snapshot.structure, &roots[0])?
        .parent_id()
        .map(str::to_owned);
    if roots.iter().any(|id| {
        snapshot.structure[index.position_by_structure_node_id[id]].parent_id() != parent.as_deref()
    }) {
        return Err("Reorder requires roots with the same parent".into());
    }
    for id in &roots {
        require_editable(&snapshot.structure, &index, id)?;
    }
    let selected: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let mut sequence = index.children_by_parent_id[&parent].clone();
    match direction {
        ReorderDirection::Front => {
            let mut picked = sequence
                .iter()
                .filter(|id| selected.contains(id.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            sequence.retain(|id| !selected.contains(id.as_str()));
            sequence.append(&mut picked);
        }
        ReorderDirection::Back => {
            let mut picked = sequence
                .iter()
                .filter(|id| selected.contains(id.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            sequence.retain(|id| !selected.contains(id.as_str()));
            picked.append(&mut sequence);
            sequence = picked;
        }
        ReorderDirection::Forward => {
            for index in (0..sequence.len().saturating_sub(1)).rev() {
                if selected.contains(sequence[index].as_str())
                    && !selected.contains(sequence[index + 1].as_str())
                {
                    sequence.swap(index, index + 1);
                }
            }
        }
        ReorderDirection::Backward => {
            for index in 1..sequence.len() {
                if selected.contains(sequence[index].as_str())
                    && !selected.contains(sequence[index - 1].as_str())
                {
                    sequence.swap(index, index - 1);
                }
            }
        }
    }
    let mut sequences = index.children_by_parent_id.clone();
    sequences.insert(parent, sequence);
    apply_sequences(&mut snapshot.structure, &sequences)?;
    Ok(roots)
}

pub(in crate::spaces::structure) fn auto_layout(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_ids: &[String],
    direction: &AutoLayoutDirection,
    gap: f64,
) -> Result<Vec<String>, String> {
    if !gap.is_finite() || gap < 0.0 {
        return Err("Auto-layout gap must be nonnegative".into());
    }
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let roots = effective_roots(&snapshot.structure, &index, structure_node_ids)?;
    if roots.is_empty() {
        return Err("Auto-layout requires selected roots".into());
    }
    let parent = index.node(&snapshot.structure, &roots[0])?.parent_id();
    if roots
        .iter()
        .any(|id| snapshot.structure[index.position_by_structure_node_id[id]].parent_id() != parent)
    {
        return Err("Auto-layout requires roots with the same parent".into());
    }
    let mut measures = roots
        .iter()
        .map(|id| {
            Ok((
                id.clone(),
                root_bounds(&snapshot.structure, &snapshot.layouts, &index, id)?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    measures
        .sort_by_key(|(id, _)| snapshot.structure[index.position_by_structure_node_id[id]].order());
    let start = measures
        .iter()
        .fold(None, |area, (_, bounds)| Some(merge(area, *bounds)))
        .ok_or("Auto-layout requires geometry")?;
    let mut cursor = if matches!(direction, AutoLayoutDirection::Right) {
        start.left
    } else {
        start.top
    };
    let mut deltas = Vec::new();
    for (id, bounds) in &measures {
        require_editable(&snapshot.structure, &index, id)?;
        let (dx, dy) = match direction {
            AutoLayoutDirection::Right => {
                let value = (cursor - bounds.left, start.top - bounds.top);
                cursor += bounds.right - bounds.left + gap;
                value
            }
            AutoLayoutDirection::Down => {
                let value = (start.left - bounds.left, cursor - bounds.top);
                cursor += bounds.bottom - bounds.top + gap;
                value
            }
        };
        deltas.push((id.clone(), dx, dy));
    }
    let mut updates = HashMap::with_capacity(deltas.len());
    for (id, dx, dy) in deltas {
        updates.insert(id.clone(), moved_layout(snapshot, &index, &id, dx, dy)?);
    }
    replace_layouts(&mut snapshot.layouts, updates);
    Ok(roots)
}
