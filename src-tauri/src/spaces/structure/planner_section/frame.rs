use super::common::{frame_for_bounds, replace_layouts, shift_local};
use crate::spaces::structure::edit_contract::SpaceRect;
use crate::spaces::structure::planner_bounds::bounds_for_roots;
use crate::spaces::structure::planner_spatial::layout_keeping_world;
use crate::spaces::structure::planner_support::{
    apply_sequences, effective_roots, require_editable, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use crate::spaces::structure::{Point, Size, StructureLayout};
use std::collections::{HashMap, HashSet};

pub(in crate::spaces::structure) fn resize(
    snapshot: &mut SpaceStructureSnapshot,
    id: &str,
    frame: &SpaceRect,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let structure_node = index.node(&snapshot.structure, id)?;
    if structure_node.kind_name() != "section" {
        return Err(format!("Section not found: {id}"));
    }
    require_editable(&snapshot.structure, &index, id)?;
    let layout_index = index.layout[id];
    let StructureLayout::Section { position, .. } = &snapshot.layouts[layout_index] else {
        return Err(format!("Section layout missing: {id}"));
    };
    let dx = frame.x - position.x;
    let dy = frame.y - position.y;
    let children: HashSet<String> = index
        .children
        .get(&Some(id.to_owned()))
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    for layout in &mut snapshot.layouts {
        if layout.structure_node_id() == id {
            if let StructureLayout::Section { position, size, .. } = layout {
                *position = Point {
                    x: frame.x,
                    y: frame.y,
                };
                *size = Size {
                    width: frame.width,
                    height: frame.height,
                };
            }
        } else if children.contains(layout.structure_node_id()) {
            shift_local(layout, -dx, -dy)?;
        }
    }
    Ok(vec![id.to_owned()])
}

pub(in crate::spaces::structure) fn fit(snapshot: &mut SpaceStructureSnapshot, id: &str) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    if index.node(&snapshot.structure, id)?.kind_name() != "section" {
        return Err(format!("Section not found: {id}"));
    }
    require_editable(&snapshot.structure, &index, id)?;
    let children = index
        .children
        .get(&Some(id.to_owned()))
        .cloned()
        .unwrap_or_default();
    let bounds = bounds_for_roots(&snapshot.structure, &snapshot.layouts, &index, &children)?
        .ok_or("Cannot fit an empty Section")?;
    resize(snapshot, id, &frame_for_bounds(bounds))
}

pub(in crate::spaces::structure) fn dissolve(
    snapshot: &mut SpaceStructureSnapshot,
    requested: &[String],
) -> Result<Vec<String>, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    let sections = effective_roots(&original_structure, &index, requested)?;
    if sections.is_empty()
        || sections
            .iter()
            .any(|id| index.node(&original_structure, id).unwrap().kind_name() != "section")
    {
        return Err("Dissolve requires selected Section roots".into());
    }
    for id in &sections {
        require_editable(&original_structure, &index, id)?;
    }
    let removed: HashSet<&str> = sections.iter().map(String::as_str).collect();
    let mut promoted = Vec::new();
    let mut updates = HashMap::new();
    for id in &sections {
        for child in index
            .children
            .get(&Some(id.clone()))
            .cloned()
            .unwrap_or_default()
        {
            promoted.push(child.clone());
            updates.insert(
                child.clone(),
                layout_keeping_world(&original_structure, &original_layouts, &index, &child, None)?,
            );
        }
    }
    let roots = index
        .children
        .get(&None)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .flat_map(|id| {
            if removed.contains(id.as_str()) {
                index.children.get(&Some(id)).cloned().unwrap_or_default()
            } else {
                vec![id]
            }
        })
        .collect();
    snapshot
        .structure
        .retain(|structure_node| !removed.contains(structure_node.id()));
    snapshot
        .layouts
        .retain(|layout| !removed.contains(layout.structure_node_id()));
    replace_layouts(&mut snapshot.layouts, &updates);
    apply_sequences(&mut snapshot.structure, &HashMap::from([(None, roots)]))?;
    Ok(promoted)
}
