use super::{context::Context, draft::Draft, contract::{Axis, Alignment}, movement};
use crate::spaces::canvas_state::{contract::StructurePoint, geometry::{self, Bounds}};
pub(super) fn align(context: &Context<'_>, draft: &mut Draft, ids: &[String], axis: Axis, alignment: Alignment) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?;
    if roots.len() < 2 { return Err("Alignment requires at least two operation roots".into()); } context.editable(&roots)?;
    let bounds: Vec<_> = roots.iter().map(|id| context.projection.bounds[id]).collect();
    let all = geometry::union_bounds(&bounds).ok_or("No arrangement bounds")?;
    let fraction = match alignment { Alignment::Start => 0.0, Alignment::Center => 0.5, Alignment::End => 1.0 };
    let target = position(all, axis) + size(all, axis) * fraction;
    for id in &roots { let rect = context.projection.bounds[id]; move_axis(context, draft, id, axis, target - position(rect, axis) - size(rect, axis) * fraction)?; }
    Ok(roots)
}
pub(super) fn distribute(context: &Context<'_>, draft: &mut Draft, ids: &[String], axis: Axis) -> Result<Vec<String>, String> {
    let mut roots = context.roots(ids, false)?;
    if roots.len() < 3 { return Err("Distribution requires at least three operation roots".into()); } context.editable(&roots)?;
    let order: std::collections::HashMap<_,_> = context.projection.order.iter().enumerate().map(|(index,id)| (id.as_str(),index)).collect();
    roots.sort_by(|left,right| position(context.projection.bounds[left],axis).total_cmp(&position(context.projection.bounds[right],axis)).then(order[left.as_str()].cmp(&order[right.as_str()])));
    let first = context.projection.bounds[&roots[0]]; let last = context.projection.bounds[roots.last().ok_or("No selection")?];
    let total: f64 = roots.iter().map(|id| size(context.projection.bounds[id], axis)).sum();
    let gap = (position(last, axis) + size(last, axis) - position(first, axis) - total) / (roots.len() - 1) as f64;
    let mut cursor = position(first, axis);
    for (index,id) in roots.iter().enumerate() {
        let rect = context.projection.bounds[id];
        if index > 0 && index + 1 < roots.len() { move_axis(context, draft, id, axis, cursor - position(rect, axis))?; }
        cursor += size(rect, axis) + gap;
    }
    Ok(roots)
}
fn move_axis(context: &Context<'_>, draft: &mut Draft, id: &str, axis: Axis, delta: f64) -> Result<(), String> {
    movement::move_root(context, draft, id, match axis { Axis::X => StructurePoint { x: delta, y: 0.0 }, Axis::Y => StructurePoint { x: 0.0, y: delta } })
}
fn position(bounds: Bounds, axis: Axis) -> f64 { match axis { Axis::X => bounds.x, Axis::Y => bounds.y } }
fn size(bounds: Bounds, axis: Axis) -> f64 { match axis { Axis::X => bounds.width, Axis::Y => bounds.height } }
