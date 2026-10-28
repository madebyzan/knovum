use std::collections::HashSet;

use crate::spaces::structure::edit_contract::{AutoLayoutFlow, ReorderDirection};
use crate::spaces::structure::planner::bounds::{merge, root_bounds};
use crate::spaces::structure::planner::packing::{place, reading_order, squarest_table, table};
use crate::spaces::structure::planner::section::require_freely_arranged;
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
    flow: AutoLayoutFlow,
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
    require_freely_arranged(snapshot, &index, &roots)?;
    for id in &roots {
        require_editable(&snapshot.structure, &index, id)?;
    }
    let bounds = roots
        .iter()
        .map(|id| root_bounds(&snapshot.structure, &snapshot.layouts, &index, id))
        .collect::<Result<Vec<_>, String>>()?;
    let order = reading_order(&bounds);
    let sizes = order
        .iter()
        .map(|root| {
            let bounds = &bounds[*root];
            (bounds.right - bounds.left, bounds.bottom - bounds.top)
        })
        .collect::<Vec<_>>();
    let packed = match flow {
        AutoLayoutFlow::Row => table(&sizes, sizes.len(), gap),
        AutoLayoutFlow::Column => table(&sizes, 1, gap),
        AutoLayoutFlow::Grid => squarest_table(&sizes, gap),
    };
    let start = bounds
        .iter()
        .fold(None, |area, bounds| Some(merge(area, *bounds)))
        .ok_or("Auto-layout requires geometry")?;
    place(
        snapshot,
        &index,
        &roots,
        &bounds,
        &order,
        &packed,
        (start.left, start.top),
    )?;
    Ok(roots)
}
