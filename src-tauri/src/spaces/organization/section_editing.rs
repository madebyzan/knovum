use std::collections::HashSet;
use super::{context::Context, draft::Draft, section_creation};
use crate::spaces::canvas_state::{contract::{StructureKind, StructureLayout, StructurePoint, StructureSize, StructureTransform}, geometry::{self, Bounds}};
pub(super) fn resize(context: &Context<'_>, draft: &mut Draft, id: &str, bounds: Bounds) -> Result<Vec<String>, String> {
    context.editable(&[id.into()])?;
    match context.layouts.get(id) {
        Some(StructureLayout::Section { collapsed: false, .. }) => {},
        Some(StructureLayout::Section { .. }) => return Err("Expand the Section before resizing its region".into()),
        _ => return Err("This operation requires a Section".into()),
    }
    let inverse = geometry::inverse(StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
    for child in context.children.get(&Some(id)).into_iter().flatten() { draft.set_transform(child, geometry::compose(inverse, context.projection.world[child]))?; }
    if let Some(StructureLayout::Section { position, size, .. }) = draft.layouts.get_mut(id) {
        *position = StructurePoint { x: bounds.x, y: bounds.y }; *size = StructureSize { width: bounds.width, height: bounds.height };
    }
    Ok(vec![id.into()])
}
pub(super) fn fit(context: &Context<'_>, draft: &mut Draft, id: &str) -> Result<Vec<String>, String> {
    if context.item(id)?.kind != StructureKind::Section { return Err("Only a Section has a fit-to-contents frame".into()); }
    let ids = context.children.get(&Some(id)).cloned().unwrap_or_default();
    let bounds = context.bounds_in_parent(&ids, None)?.ok_or("An empty Section has no contents to fit")?;
    resize(context, draft, id, section_creation::around(bounds))
}
pub(super) fn collapse(context: &Context<'_>, draft: &mut Draft, id: &str, collapsed: bool, selection: &[String]) -> Result<Vec<String>, String> {
    match draft.layouts.get_mut(id) {
        Some(StructureLayout::Section { collapsed: value, .. }) => *value = collapsed,
        _ => return Err("Only a Section can be collapsed".into()),
    }
    if !collapsed { return Ok(selection.to_vec()); }
    let hidden: HashSet<_> = context.subtree(&[id.into()])?.into_iter().filter(|item| item != id).collect();
    let mut selected: Vec<_> = selection.iter().filter(|item| !hidden.contains(*item)).cloned().collect();
    if selection.iter().any(|item| hidden.contains(item)) && !selected.iter().any(|item| item == id) { selected.push(id.into()); }
    Ok(selected)
}
