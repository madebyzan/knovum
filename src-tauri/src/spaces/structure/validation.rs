use std::collections::{HashMap, HashSet};

use super::contract::{StructureLayout, StructureNode};
use super::geometry::{validate_layout, validate_world_transform};

const MAX_STRUCTURE_NODES: usize = 100_000;
const MAX_DEPTH: usize = 64;

pub(super) fn validate_id(label: &str, id: &str) -> Result<(), String> {
    if id.is_empty() || id.encode_utf16().count() > 128 {
        return Err(format!("{label} must contain 1 to 128 characters"));
    }
    Ok(())
}

pub(crate) fn validate_space_snapshot(
    structure: &[StructureNode],
    layouts: &[StructureLayout],
) -> Result<(), String> {
    if structure.len() > MAX_STRUCTURE_NODES {
        return Err("Space structure exceeds 100000 structure".into());
    }
    let mut by_id = HashMap::with_capacity(structure.len());
    let mut content_ids = HashSet::new();
    let mut sibling_orders: HashMap<Option<&str>, Vec<u32>> = HashMap::new();
    for structure_node in structure {
        if !valid_id(structure_node.id())
            || by_id.insert(structure_node.id(), structure_node).is_some()
        {
            return Err(format!(
                "Invalid or duplicate structure node ID: {}",
                structure_node.id()
            ));
        }
        if let StructureNode::Content { node_id, .. } = structure_node {
            if !valid_id(node_id) || !content_ids.insert(node_id.as_str()) {
                return Err(format!("Invalid or duplicate content node ID: {node_id}"));
            }
        }
        if let StructureNode::Group { title, .. } | StructureNode::Section { title, .. } =
            structure_node
        {
            let trimmed = title.trim();
            if trimmed.is_empty() || trimmed.encode_utf16().count() > 512 {
                return Err(format!(
                    "Container {} requires a title",
                    structure_node.id()
                ));
            }
        }
        if structure_node.order() > MAX_STRUCTURE_NODES as u32 {
            return Err(format!(
                "Structure order exceeds limit for {}",
                structure_node.id()
            ));
        }
        sibling_orders
            .entry(structure_node.parent_id())
            .or_default()
            .push(structure_node.order());
    }
    for (parent_id, orders) in &mut sibling_orders {
        orders.sort_unstable();
        if orders
            .iter()
            .enumerate()
            .any(|(index, order)| *order as usize != index)
        {
            return Err(format!("Noncanonical sibling order under {parent_id:?}"));
        }
    }
    for structure_node in structure {
        validate_parent_chain(structure_node, &by_id)?;
    }
    for structure_node in structure {
        if matches!(structure_node, StructureNode::Group { .. })
            && !sibling_orders.contains_key(&Some(structure_node.id()))
        {
            return Err(format!("Empty Group: {}", structure_node.id()));
        }
    }
    let mut layout_by_id = HashMap::with_capacity(layouts.len());
    for layout in layouts {
        let id = layout.structure_node_id();
        if !valid_id(id) || layout_by_id.insert(id, layout).is_some() {
            return Err(format!(
                "Invalid or duplicate structure_node layout ID: {id}"
            ));
        }
        let structure_node = by_id
            .get(id)
            .ok_or_else(|| format!("Layout {id} has no structure node"))?;
        if structure_node.kind_name() != layout.kind_name() {
            return Err(format!("Structure and layout kinds differ for {id}"));
        }
        validate_layout(layout)?;
    }
    if layout_by_id.len() != by_id.len() {
        return Err("Structure and layout structure_node ID sets differ".into());
    }
    for structure_node in structure {
        validate_world_transform(structure_node, &by_id, &layout_by_id)?;
    }
    Ok(())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.encode_utf16().count() <= 128
}

pub(crate) fn validate_content_coverage(
    structure: &[StructureNode],
    active_node_ids: &HashSet<String>,
) -> Result<(), String> {
    let content_ids: HashSet<&str> = structure
        .iter()
        .filter_map(|structure_node| match structure_node {
            StructureNode::Content { node_id, .. } => Some(node_id.as_str()),
            _ => None,
        })
        .collect();
    if content_ids.len() != active_node_ids.len()
        || active_node_ids
            .iter()
            .any(|id| !content_ids.contains(id.as_str()))
    {
        return Err("Active content nodes and structure content references differ".into());
    }
    Ok(())
}

fn validate_parent_chain(
    structure_node: &StructureNode,
    by_id: &HashMap<&str, &StructureNode>,
) -> Result<(), String> {
    let mut current = structure_node;
    let mut depth = 1;
    while let Some(parent_id) = current.parent_id() {
        let parent = by_id
            .get(parent_id)
            .ok_or_else(|| format!("Missing parent {parent_id} for {}", structure_node.id()))?;
        if matches!(current, StructureNode::Section { .. })
            || matches!(parent, StructureNode::Content { .. })
        {
            return Err(format!("Invalid parent type for {}", current.id()));
        }
        depth += 1;
        if depth > MAX_DEPTH {
            return Err(format!(
                "Structure depth or cycle exceeds {MAX_DEPTH} for {}",
                structure_node.id()
            ));
        }
        current = parent;
    }
    Ok(())
}
