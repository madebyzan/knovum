use std::collections::HashSet;
use super::{context::Context, draft::{Draft, title}};
use crate::spaces::canvas_state::{contract::{StructureItem, StructureKind, StructureLayout, StructureTransform}, geometry};
pub(super) fn group(context: &Context<'_>, draft: &mut Draft, ids: &[String], id: &str, name: &str) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?;
    if roots.len() < 2 || roots.iter().any(|id| context.items[id.as_str()].kind == StructureKind::Section) { return Err("Combine at least two sibling contents or Groups".into()); }
    let parent = context.shared_parent(&roots)?; context.editable(&roots)?; draft.assert_new(id)?;
    let bounds = context.bounds_in_parent(&roots, parent.as_deref())?.ok_or("Selection has no bounds")?;
    let transform = StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() };
    let siblings = replacement_siblings(context, &roots, id)?;
    draft.items.insert(id.into(), StructureItem { id: id.into(), kind: StructureKind::Group, parent_id: parent.clone(), order: 0, locked: false, node_id: None, title: Some(title(name)?) });
    draft.layouts.insert(id.into(), StructureLayout::Group { item_id: id.into(), transform });
    let inverse = geometry::inverse(transform);
    for root in &roots { draft.set_transform(root, geometry::compose(inverse, context.layouts[root.as_str()].local()))?; }
    draft.set_siblings(Some(id), &roots)?; draft.set_siblings(parent.as_deref(), &siblings)?;
    Ok(vec![id.into()])
}
pub(super) fn replacement_siblings(context: &Context<'_>, roots: &[String], id: &str) -> Result<Vec<String>, String> {
    let parent = context.shared_parent(roots)?;
    let selected: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let front = roots.iter().map(|id| context.items[id.as_str()].order).max().ok_or("Selection is empty")?;
    let mut siblings: Vec<_> = context.children.get(&parent.as_deref()).into_iter().flatten().filter(|id| !selected.contains(id.as_str())).cloned().collect();
    let index = siblings.iter().filter(|id| context.items[id.as_str()].order < front).count();
    siblings.insert(index, id.into()); Ok(siblings)
}
pub(super) fn dissolve(context: &Context<'_>, draft: &mut Draft, id: &str) -> Result<Vec<String>, String> {
    let item = context.item(id)?;
    if item.kind == StructureKind::Content { return Err("Only a container can be dissolved".into()); }
    context.editable(&[id.into()])?;
    let children = context.children.get(&Some(id)).cloned().unwrap_or_default();
    let inverse = geometry::inverse(context.parent_world(item.parent_id.as_deref())?);
    for child in &children { draft.set_transform(child, geometry::compose(inverse, context.projection.world[child]))?; }
    let mut siblings = draft.siblings(item.parent_id.as_deref());
    let index = siblings.iter().position(|candidate| candidate == id).ok_or("Container has no sibling position")?;
    siblings.splice(index..index+1, children.iter().cloned());
    draft.remove(id); draft.set_siblings(item.parent_id.as_deref(), &siblings)?;
    Ok(children)
}
