use super::common::{append_section, frame_for_bounds, replace_layouts, require_new};
use crate::spaces::structure::planner_bounds::bounds_for_roots;
use crate::spaces::structure::planner_spatial::layout_keeping_world;
use crate::spaces::structure::planner_support::{
    apply_sequences, effective_roots, require_editable, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use std::collections::{HashMap, HashSet};

pub(in crate::spaces::structure) fn wrap(
    snapshot: &mut SpaceStructureSnapshot,
    section_id: &str,
    title: &str,
    requested: &[String],
) -> Result<Vec<String>, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    require_new(&index, section_id)?;
    let roots = effective_roots(&original_structure, &index, requested)?;
    if roots.is_empty()
        || roots
            .iter()
            .any(|id| index.node(&original_structure, id).unwrap().kind_name() == "section")
    {
        return Err("Wrapping requires Content or Group roots".into());
    }
    let parent = index
        .node(&original_structure, &roots[0])?
        .parent_id()
        .map(str::to_owned);
    if roots
        .iter()
        .any(|id| index.node(&original_structure, id).unwrap().parent_id() != parent.as_deref())
    {
        return Err("Wrapping requires the same direct parent".into());
    }
    if let Some(parent_id) = &parent {
        if index.node(&original_structure, parent_id)?.kind_name() != "section" {
            return Err("Wrapping cannot extract structure from a Group".into());
        }
    }
    for id in &roots {
        require_editable(&original_structure, &index, id)?;
    }
    let bounds = bounds_for_roots(&original_structure, &original_layouts, &index, &roots)?
        .ok_or("Selection has no content geometry")?;
    let frame = frame_for_bounds(bounds);
    append_section(snapshot, section_id, title, &frame, 0);
    let temporary = Index::new(&snapshot.structure, &snapshot.layouts);
    let updates = roots
        .iter()
        .map(|id| {
            Ok((
                id.clone(),
                layout_keeping_world(
                    &snapshot.structure,
                    &snapshot.layouts,
                    &temporary,
                    id,
                    Some(section_id),
                )?,
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    replace_layouts(&mut snapshot.layouts, &updates);
    let selected: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let source = index.children.get(&parent).cloned().unwrap_or_default();
    let members = source
        .iter()
        .filter(|id| selected.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let root_ids = index
        .children
        .get(&None)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .flat_map(|id| {
            if parent.is_none() && members.first() == Some(&id) {
                vec![section_id.to_owned()]
            } else if parent.is_none() && selected.contains(id.as_str()) {
                vec![]
            } else if parent.as_deref() == Some(id.as_str()) {
                vec![id, section_id.to_owned()]
            } else {
                vec![id]
            }
        })
        .collect();
    let mut sequences = HashMap::from([(None, root_ids), (Some(section_id.to_owned()), members)]);
    if parent.is_some() {
        sequences.insert(
            parent,
            source
                .into_iter()
                .filter(|id| !selected.contains(id.as_str()))
                .collect(),
        );
    }
    apply_sequences(&mut snapshot.structure, &sequences)?;
    Ok(vec![section_id.to_owned()])
}
