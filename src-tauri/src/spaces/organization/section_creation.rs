use std::collections::HashSet;
use super::{context::Context, draft::{Draft, title}, grouping::replacement_siblings};
use crate::spaces::canvas_state::{contract::{StructureItem, StructureKind, StructureLayout, StructurePoint, StructureSize, StructureTransform}, geometry::{self, Bounds}};
pub(super) fn around(bounds: Bounds) -> Bounds { Bounds { x: bounds.x - 32.0, y: bounds.y - 56.0, width: bounds.width + 64.0, height: bounds.height + 88.0 } }
pub(super) fn draw(context: &Context<'_>, draft: &mut Draft, id: &str, title: &str, bounds: Bounds, ids: &[String]) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, true)?; context.editable(&roots)?;
    for id in &roots {
        let item = context.item(id)?;
        if item.parent_id.is_some() || item.kind == StructureKind::Section { return Err("Drawing can adopt only root-level content or Groups".into()); }
        if !contains(bounds, context.projection.bounds[id]) { return Err("A drawn Section must fully enclose each adopted operation unit".into()); }
    }
    create(context, draft, id, title, bounds, &roots)
}
pub(super) fn wrap(context: &Context<'_>, draft: &mut Draft, id: &str, title: &str, ids: &[String]) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?; let parent = context.shared_parent(&roots)?;
    if roots.iter().any(|id| context.items[id.as_str()].kind == StructureKind::Section) || parent.as_deref().is_some_and(|id| context.items[id].kind != StructureKind::Section) { return Err("Wrap sibling contents or Groups at the root or inside one Section".into()); }
    context.editable(&roots)?;
    let bounds = context.bounds_in_parent(&roots, None)?.ok_or("No content geometry")?;
    create(context, draft, id, title, around(bounds), &roots)
}
pub(super) fn create(context: &Context<'_>, draft: &mut Draft, id: &str, name: &str, bounds: Bounds, roots: &[String]) -> Result<Vec<String>, String> {
    draft.assert_new(id)?;
    let parent = if roots.is_empty() { None } else { context.shared_parent(roots)? };
    let mut siblings = if !roots.is_empty() && parent.is_none() { replacement_siblings(context, roots, id)? } else { draft.siblings(None) };
    if roots.is_empty() { siblings.push(id.into()); }
    else if let Some(parent_id) = &parent {
        let index = siblings.iter().position(|item| item == parent_id).ok_or("Source Section has no root position")?;
        siblings.insert(index + 1, id.into());
    }
    draft.items.insert(id.into(), StructureItem { id: id.into(), kind: StructureKind::Section, parent_id: None, order: 0, locked: false, node_id: None, title: Some(title(name)?) });
    draft.layouts.insert(id.into(), StructureLayout::Section { item_id: id.into(), position: StructurePoint { x: bounds.x, y: bounds.y }, size: StructureSize { width: bounds.width, height: bounds.height }, collapsed: false, appearance: None });
    let inverse = geometry::inverse(StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
    for root in roots { draft.set_transform(root, geometry::compose(inverse, context.projection.world[root]))?; }
    draft.set_siblings(Some(id), roots)?;
    if parent.is_some() { draft.normalize(&HashSet::from([parent]))?; }
    draft.set_siblings(None, &siblings)?;
    Ok(vec![id.into()])
}
fn contains(outer: Bounds, inner: Bounds) -> bool { inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.width <= outer.x + outer.width && inner.y + inner.height <= outer.y + outer.height }
