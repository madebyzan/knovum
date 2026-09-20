use std::collections::HashSet;
use super::{context::Context, draft::Draft, movement};
use crate::spaces::canvas_state::{contract::{StructureKind, StructurePoint, StructureTransform}, geometry};
pub(super) fn reparent(context: &Context<'_>, draft: &mut Draft, ids: &[String], parent: Option<&str>, insertion: Option<usize>, delta: StructurePoint, dropping: bool) -> Result<Vec<String>, String> {
    let mut roots = context.roots(ids, false)?; context.editable(&roots)?;
    if dropping && roots.iter().all(|id| context.items[id.as_str()].parent_id.as_deref() == parent) { return movement::move_items(context, draft, &roots, delta); }
    validate_destination(context, &roots, parent)?;
    if !geometry::coordinate(delta.x) || !geometry::coordinate(delta.y) { return Err("Invalid drop displacement".into()); }
    let order: std::collections::HashMap<_,_> = context.projection.order.iter().enumerate().map(|(index,id)| (id.as_str(),index)).collect();
    roots.sort_by_key(|id| order[id.as_str()]);
    let chosen: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let mut siblings: Vec<_> = draft.siblings(parent).into_iter().filter(|id| !chosen.contains(id.as_str())).collect();
    let position = insertion.unwrap_or(siblings.len());
    if position > siblings.len() { return Err("Insertion index is outside the remaining sibling list".into()); }
    let inverse = geometry::inverse(context.parent_world(parent)?);
    let old_parents: HashSet<_> = roots.iter().map(|id| context.items[id.as_str()].parent_id.clone()).filter(|id| id.as_deref() != parent).collect();
    for id in &roots {
        let world = context.projection.world[id];
        draft.set_transform(id, geometry::compose(inverse, StructureTransform { x: world.x + delta.x, y: world.y + delta.y, ..world }))?;
        draft.items.get_mut(id).ok_or("Moving item is unavailable")?.parent_id = parent.map(str::to_string);
    }
    draft.normalize(&old_parents)?; siblings.splice(position..position, roots.iter().cloned());
    draft.set_siblings(parent, &siblings)?; draft.prune_empty_groups()?; Ok(roots)
}
fn validate_destination(context: &Context<'_>, roots: &[String], parent: Option<&str>) -> Result<(), String> {
    if let Some(parent) = parent {
        let target = context.item(parent)?;
        if target.kind == StructureKind::Content { return Err("Content cannot contain other items".into()); }
        context.editable(&[parent.into()])?;
        if roots.iter().any(|id| context.items[id.as_str()].kind == StructureKind::Section) { return Err("Sections can only belong to the Space root".into()); }
        if context.subtree(roots)?.iter().any(|id| id == parent) { return Err("An item cannot be moved into its own subtree".into()); }
    }
    Ok(())
}
