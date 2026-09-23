use super::common::{append_section, replace_layouts, require_new};
use crate::spaces::structure::edit_contract::SpaceRect;
use crate::spaces::structure::planner_bounds::bounds_for_roots;
use crate::spaces::structure::planner_spatial::layout_keeping_world;
use crate::spaces::structure::planner_support::{apply_sequences, require_editable, Index};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use std::collections::{HashMap, HashSet};

pub(in crate::spaces::structure) fn draw(
    snapshot: &mut SpaceStructureSnapshot,
    section_id: &str,
    title: &str,
    frame: &SpaceRect,
    expected: &[String],
) -> Result<Vec<String>, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    require_new(&index, section_id)?;
    let roots = index.children.get(&None).cloned().unwrap_or_default();
    let mut candidates = Vec::new();
    for id in &roots {
        if index.node(&original_structure, id)?.kind_name() == "section" {
            continue;
        }
        if let Some(bounds) = bounds_for_roots(
            &original_structure,
            &original_layouts,
            &index,
            std::slice::from_ref(id),
        )? {
            if bounds.left >= frame.x - 0.5
                && bounds.top >= frame.y - 0.5
                && bounds.right <= frame.x + frame.width + 0.5
                && bounds.bottom <= frame.y + frame.height + 0.5
            {
                candidates.push(id.clone());
            }
        }
    }
    if expected.len() != candidates.len()
        || expected.iter().collect::<HashSet<_>>() != candidates.iter().collect::<HashSet<_>>()
    {
        return Err("Section draw candidates changed".into());
    }
    for id in &candidates {
        require_editable(&original_structure, &index, id)?;
    }
    let order = candidates
        .first()
        .map(|id| index.node(&original_structure, id).unwrap().order())
        .unwrap_or(roots.len() as u32);
    append_section(snapshot, section_id, title, frame, order);
    let temporary = Index::new(&snapshot.structure, &snapshot.layouts);
    let updates = candidates
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
    let selected: HashSet<&str> = candidates.iter().map(String::as_str).collect();
    let mut root_sequence = roots
        .iter()
        .flat_map(|id| {
            if candidates.first() == Some(id) {
                vec![section_id.to_owned()]
            } else if selected.contains(id.as_str()) {
                vec![]
            } else {
                vec![id.clone()]
            }
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        root_sequence.push(section_id.to_owned());
    }
    apply_sequences(
        &mut snapshot.structure,
        &HashMap::from([
            (None, root_sequence),
            (Some(section_id.to_owned()), candidates),
        ]),
    )?;
    Ok(vec![section_id.to_owned()])
}
