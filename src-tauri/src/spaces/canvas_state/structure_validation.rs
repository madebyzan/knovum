use std::collections::{HashMap, HashSet};
use super::contract::{SpaceStructure, StructureItem, StructureKind};
pub(super) const MAX_ITEMS: usize = 100_000;
pub(super) const MAX_DEPTH: usize = 64;
pub(super) fn valid_id(id: &str) -> bool { !id.is_empty() && id.encode_utf16().count() <= 128 && !id.chars().any(|character| character.is_whitespace() || character as u32 == 0xfeff) }
pub(super) fn validate(structure: &SpaceStructure) -> Result<(), String> {
    if structure.version != 1 || structure.items.len() > MAX_ITEMS { return Err("Unsupported or oversized Space structure".into()); }
    let mut items = HashMap::with_capacity(structure.items.len());
    let mut children: HashMap<Option<&str>, Vec<&StructureItem>> = HashMap::new();
    for item in &structure.items {
        validate_item(item)?;
        if items.insert(item.id.as_str(), item).is_some() { return Err("Structure identities must be unique".into()); }
        children.entry(item.parent_id.as_deref()).or_default().push(item);
    }
    for item in &structure.items {
        if let Some(parent) = item.parent_id.as_deref() {
            if items.get(parent).is_none_or(|item| item.kind == StructureKind::Content) { return Err("Every parent must be an existing container in this Space".into()); }
        }
        if item.kind == StructureKind::Group && !children.contains_key(&Some(item.id.as_str())) { return Err("A committed Group must not be empty".into()); }
    }
    for siblings in children.values_mut() {
        siblings.sort_by_key(|item| item.order);
        if siblings.iter().enumerate().any(|(index, item)| index != item.order as usize) { return Err("Sibling order must be unique and contiguous".into()); }
    }
    let mut pending: Vec<(&StructureItem, usize)> = children.get(&None).into_iter().flatten().map(|item| (*item, 1)).collect();
    let mut visited = HashSet::new();
    while let Some((item, depth)) = pending.pop() {
        if depth > MAX_DEPTH { return Err("Structure nesting exceeds 64 levels".into()); }
        if !visited.insert(item.id.as_str()) { return Err("Structure contains a cycle".into()); }
        pending.extend(children.get(&Some(item.id.as_str())).into_iter().flatten().map(|child| (*child, depth + 1)));
    }
    if visited.len() != items.len() { return Err("Structure contains an unreachable cycle".into()); }
    Ok(())
}
fn validate_item(item: &StructureItem) -> Result<(), String> {
    if !valid_id(&item.id) || item.parent_id.as_deref().is_some_and(|id| !valid_id(id)) || item.order as usize >= MAX_ITEMS { return Err("Invalid structure identity or order".into()); }
    match item.kind {
        StructureKind::Content => {
            if item.node_id.as_deref() != Some(item.id.as_str()) || item.title.is_some() { return Err("Content placements must use their content identity and cannot own a second title".into()); }
        }
        StructureKind::Group | StructureKind::Section => {
            let title = item.title.as_deref().ok_or("Containers require titles")?;
            if item.node_id.is_some() || title.is_empty() || title.encode_utf16().count() > 512 || title.trim_matches(|character: char| character.is_whitespace() || character as u32 == 0xfeff) != title { return Err("Container titles must be trimmed and have 1 to 512 characters".into()); }
            if item.kind == StructureKind::Section && item.parent_id.is_some() { return Err("Sections belong directly to the Space root".into()); }
        }
    }
    Ok(())
}
