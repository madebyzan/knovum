use super::{context::Context, draft::Draft};
use crate::spaces::canvas_state::{contract::{StructureKind, StructurePoint, StructureTransform}, geometry};
pub(super) fn move_items(context: &Context<'_>, draft: &mut Draft, ids: &[String], delta: StructurePoint) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?; context.editable(&roots)?;
    if !geometry::coordinate(delta.x) || !geometry::coordinate(delta.y) { return Err("Invalid movement delta".into()); }
    for id in &roots { move_root(context, draft, id, delta)?; } Ok(roots)
}
pub(super) fn move_root(context: &Context<'_>, draft: &mut Draft, id: &str, delta: StructurePoint) -> Result<(), String> {
    if delta.x == 0.0 && delta.y == 0.0 { return Ok(()); }
    let item = context.item(id)?; let world = context.projection.world[id];
    let inverse = geometry::inverse(context.parent_world(item.parent_id.as_deref())?);
    draft.set_transform(id, geometry::compose(inverse, StructureTransform { x: world.x + delta.x, y: world.y + delta.y, ..world }))
}
pub(super) fn transform_group(context: &Context<'_>, draft: &mut Draft, id: &str, world: StructureTransform) -> Result<Vec<String>, String> {
    let item = context.item(id)?;
    if item.kind != StructureKind::Group { return Err("Only a Group supports aggregate rotation and scaling".into()); }
    context.editable(&[id.into()])?;
    if !geometry::valid_transform(world) { return Err("Invalid Group world transform".into()); }
    draft.set_transform(id, geometry::compose(geometry::inverse(context.parent_world(item.parent_id.as_deref())?), world))?;
    Ok(vec![id.into()])
}
