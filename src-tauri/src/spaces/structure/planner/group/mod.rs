use std::collections::{HashMap, HashSet};

mod dissolve;
pub(in crate::spaces::structure) use dissolve::dissolve;

use crate::spaces::structure::contracts::{LocalTransform, StructureLayout, StructureNode};
use crate::spaces::structure::planner::spatial::{layout_keeping_world, local_transform};
use crate::spaces::structure::planner::tree_policy::{
    apply_sequences, effective_roots, require_editable, Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;

pub(in crate::spaces::structure) fn create(
    snapshot: &mut SpaceStructureSnapshot,
    group_id: &str,
    requested: &[String],
    title: &str,
) -> Result<Vec<String>, String> {
    let original_structure = snapshot.structure.clone();
    let original_layouts = snapshot.layouts.clone();
    let index = Index::new(&original_structure, &original_layouts);
    if index.positions.contains_key(group_id) {
        return Err(format!("Structure node already exists: {group_id}"));
    }
    let roots = effective_roots(&original_structure, &index, requested)?;
    if roots.len() < 2
        || roots
            .iter()
            .any(|id| index.node(&original_structure, id).unwrap().kind_name() == "section")
    {
        return Err("Grouping requires at least two Content or Group roots".into());
    }
    let parent = index
        .node(&original_structure, &roots[0])?
        .parent_id()
        .map(str::to_owned);
    if roots
        .iter()
        .any(|id| index.node(&original_structure, id).unwrap().parent_id() != parent.as_deref())
    {
        return Err("Grouping requires the same direct parent".into());
    }
    for id in &roots {
        require_editable(&original_structure, &index, id)?;
    }
    let siblings = index.children.get(&parent).cloned().unwrap_or_default();
    let selected: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let first = siblings
        .iter()
        .find(|id| selected.contains(id.as_str()))
        .ok_or("Group anchor missing")?;
    let anchor = local_transform(&original_layouts[index.layout[first]]);
    snapshot.structure.push(StructureNode::Group {
        id: group_id.to_owned(),
        parent_id: parent.clone(),
        order: index.node(&original_structure, first)?.order(),
        locked: false,
        title: title.to_owned(),
    });
    snapshot.layouts.push(StructureLayout::Group {
        structure_node_id: group_id.to_owned(),
        transform: LocalTransform {
            x: anchor.x,
            y: anchor.y,
            rotation: 0.0,
            scale: 1.0,
        },
    });
    let temporary = Index::new(&snapshot.structure, &snapshot.layouts);
    let updates: HashMap<String, StructureLayout> = roots
        .iter()
        .map(|id| {
            Ok((
                id.clone(),
                layout_keeping_world(
                    &snapshot.structure,
                    &snapshot.layouts,
                    &temporary,
                    id,
                    Some(group_id),
                )?,
            ))
        })
        .collect::<Result<_, String>>()?;
    for layout in &mut snapshot.layouts {
        if let Some(update) = updates.get(layout.structure_node_id()) {
            *layout = update.clone();
        }
    }
    let parent_sequence = siblings
        .iter()
        .flat_map(|id| {
            if id == first {
                vec![group_id.to_owned()]
            } else if selected.contains(id.as_str()) {
                vec![]
            } else {
                vec![id.clone()]
            }
        })
        .collect();
    let sequences = HashMap::from([
        (parent.clone(), parent_sequence),
        (
            Some(group_id.to_owned()),
            siblings
                .into_iter()
                .filter(|id| selected.contains(id.as_str()))
                .collect(),
        ),
    ]);
    apply_sequences(&mut snapshot.structure, &sequences)?;
    Ok(vec![group_id.to_owned()])
}
