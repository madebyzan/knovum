use crate::spaces::structure::planner::spatial::layout_keeping_world;
use crate::spaces::structure::planner::tree_policy::{
    apply_sequences, effective_roots, remove_empty_groups, require_editable, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;

pub(in crate::spaces::structure) fn reparent(
    snapshot: &mut SpaceStructureSnapshot,
    ids: &[String],
    parent: Option<&str>,
    order: u32,
) -> Result<Vec<String>, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    let roots = effective_roots(&original_structure, &index, ids)?;
    if roots.is_empty() {
        return Err("Reparent requires selected structure nodes".into());
    }
    let old_parent = index
        .node(&original_structure, &roots[0])?
        .parent_id()
        .map(str::to_owned);
    if roots
        .iter()
        .any(|id| index.node(&original_structure, id).unwrap().parent_id() != old_parent.as_deref())
    {
        return Err("Reparent requires the same direct parent".into());
    }
    if let Some(parent_id) = parent {
        let target = index.node(&original_structure, parent_id)?;
        if target.kind_name() == "content" {
            return Err("Invalid reparent target".into());
        }
        require_editable(&original_structure, &index, parent_id)?;
        if roots.iter().any(|root| {
            root == parent_id
                || crate::spaces::structure::planner::tree_policy::ancestors(
                    &original_structure,
                    &index,
                    parent_id,
                )
                .unwrap()
                .contains(root)
        }) {
            return Err("Structure cycle".into());
        }
    }
    for id in &roots {
        require_editable(&original_structure, &index, id)?;
        if index.node(&original_structure, id)?.kind_name() == "section" && parent.is_some() {
            return Err("A Section must remain at the Space root".into());
        }
    }
    let selected: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let ordered: Vec<String> = index.children[&old_parent]
        .iter()
        .filter(|id| selected.contains(id.as_str()))
        .cloned()
        .collect();
    let mut sequences = index.children.clone();
    sequences
        .entry(old_parent.clone())
        .or_default()
        .retain(|id| !selected.contains(id.as_str()));
    let destination = sequences.entry(parent.map(str::to_owned)).or_default();
    if order as usize > destination.len() {
        return Err("Invalid reparent insertion order".into());
    }
    destination.splice(order as usize..order as usize, ordered.clone());
    let updates = roots
        .iter()
        .map(|id| {
            Ok((
                id.clone(),
                layout_keeping_world(&original_structure, &original_layouts, &index, id, parent)?,
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    apply_sequences(&mut snapshot.structure, &sequences)?;
    replace_layouts(&mut snapshot.layouts, updates);
    remove_empty_groups(&mut snapshot.structure, &mut snapshot.layouts);
    normalize_orders(snapshot);
    Ok(ordered)
}

fn normalize_orders(snapshot: &mut SpaceStructureSnapshot) {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let _ = apply_sequences(&mut snapshot.structure, &index.children);
}
use std::collections::{HashMap, HashSet};

use crate::spaces::structure::planner::movement::replace_layouts;
