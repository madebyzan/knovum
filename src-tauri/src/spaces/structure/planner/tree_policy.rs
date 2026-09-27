use std::collections::{HashMap, HashSet};

use crate::spaces::structure::{StructureLayout, StructureNode};

pub(in crate::spaces::structure) struct Index {
    pub positions: HashMap<String, usize>,
    pub children: HashMap<Option<String>, Vec<String>>,
    pub layout: HashMap<String, usize>,
}

impl Index {
    pub fn new(structure: &[StructureNode], layouts: &[StructureLayout]) -> Self {
        let positions: HashMap<String, usize> = structure
            .iter()
            .enumerate()
            .map(|(index, value)| (value.id().to_owned(), index))
            .collect();
        let mut children: HashMap<Option<String>, Vec<String>> = HashMap::new();
        for value in structure {
            children
                .entry(value.parent_id().map(str::to_owned))
                .or_default()
                .push(value.id().to_owned());
        }
        for ids in children.values_mut() {
            ids.sort_by_key(|id| structure[positions[id]].order());
        }
        let layout = layouts
            .iter()
            .enumerate()
            .map(|(index, value)| (value.structure_node_id().to_owned(), index))
            .collect();
        Self {
            positions,
            children,
            layout,
        }
    }

    pub fn node<'a>(
        &self,
        structure: &'a [StructureNode],
        id: &str,
    ) -> Result<&'a StructureNode, String> {
        self.positions
            .get(id)
            .map(|index| &structure[*index])
            .ok_or_else(|| format!("Structure node not found: {id}"))
    }
}

pub(in crate::spaces::structure) fn effective_roots(
    structure: &[StructureNode],
    index: &Index,
    requested: &[String],
) -> Result<Vec<String>, String> {
    let selected: HashSet<&str> = requested.iter().map(String::as_str).collect();
    let mut added = HashSet::new();
    let mut roots = Vec::new();
    for id in requested {
        index.node(structure, id)?;
        if !added.insert(id.as_str())
            || ancestors(structure, index, id)?
                .iter()
                .any(|id| selected.contains(id.as_str()))
        {
            continue;
        }
        roots.push(id.clone());
    }
    Ok(roots)
}

pub(in crate::spaces::structure) fn ancestors(
    structure: &[StructureNode],
    index: &Index,
    id: &str,
) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut parent = index.node(structure, id)?.parent_id();
    while let Some(parent_id) = parent {
        result.push(parent_id.to_owned());
        parent = index.node(structure, parent_id)?.parent_id();
    }
    result.reverse();
    Ok(result)
}

pub(in crate::spaces::structure) fn require_editable(
    structure: &[StructureNode],
    index: &Index,
    id: &str,
) -> Result<(), String> {
    for ancestor in ancestors(structure, index, id)?
        .into_iter()
        .chain([id.to_owned()])
    {
        if index.node(structure, &ancestor)?.locked() {
            return Err(format!("Locked structure node: {id}"));
        }
    }
    Ok(())
}

pub(in crate::spaces::structure) fn set_parent(
    structure_node: &mut StructureNode,
    parent_id: Option<String>,
) {
    match structure_node {
        StructureNode::Content {
            parent_id: value, ..
        }
        | StructureNode::Group {
            parent_id: value, ..
        }
        | StructureNode::Section {
            parent_id: value, ..
        } => *value = parent_id,
    }
}

pub(in crate::spaces::structure) fn set_order(structure_node: &mut StructureNode, order: u32) {
    match structure_node {
        StructureNode::Content { order: value, .. }
        | StructureNode::Group { order: value, .. }
        | StructureNode::Section { order: value, .. } => *value = order,
    }
}

pub(in crate::spaces::structure) fn apply_sequences(
    structure: &mut [StructureNode],
    sequences: &HashMap<Option<String>, Vec<String>>,
) -> Result<(), String> {
    let positions: HashMap<&str, (&Option<String>, usize)> = sequences
        .iter()
        .flat_map(|(parent, ids)| {
            ids.iter()
                .enumerate()
                .map(move |(order, id)| (id.as_str(), (parent, order)))
        })
        .collect();
    for structure_node in structure {
        if let Some((parent, order)) = positions.get(structure_node.id()) {
            set_parent(structure_node, (*parent).clone());
            set_order(
                structure_node,
                u32::try_from(*order).map_err(|_| "Structure order overflow")?,
            );
        }
    }
    Ok(())
}

pub(in crate::spaces::structure) fn remove_empty_groups(
    structure: &mut Vec<StructureNode>,
    layouts: &mut Vec<StructureLayout>,
) -> HashSet<String> {
    let mut removed = HashSet::new();
    loop {
        let empty = structure
            .iter()
            .find(|structure_node| {
                structure_node.kind_name() == "group"
                    && !structure
                        .iter()
                        .any(|child| child.parent_id() == Some(structure_node.id()))
            })
            .map(|structure_node| structure_node.id().to_owned());
        let Some(id) = empty else { break };
        removed.insert(id.clone());
        structure.retain(|structure_node| structure_node.id() != id);
        layouts.retain(|layout| layout.structure_node_id() != id);
    }
    removed
}
