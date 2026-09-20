use super::{context::Context, draft::Draft, editor_command::FrameUpdate, movement, section_editing};
use crate::spaces::canvas_state::{contract::{StructureKind, StructureLayout, StructureTransform}, geometry::{self, Bounds}};
pub(super) fn transform(context: &Context<'_>, draft: &mut Draft, updates: &[FrameUpdate]) -> Result<Vec<String>, String> {
    let ids = validate_updates(context, updates)?;
    let roots = context.roots(&ids, false)?; context.editable(&roots)?;
    let by_id: std::collections::HashMap<_,_> = updates.iter().map(|update| (update.node_id.as_str(), update)).collect();
    for id in &roots {
        let update = by_id[id.as_str()]; let frame = &update.frame;
        let item = context.item(id)?;
        match item.kind {
            StructureKind::Section => {
                if frame.rotation != 0.0 { return Err("Sections cannot rotate".into()); }
                section_editing::resize(context, draft, id, Bounds { x: frame.x, y: frame.y, width: frame.width, height: frame.height })?;
            },
            StructureKind::Group => {
                let children = context.children.get(&Some(id.as_str())).cloned().unwrap_or_default();
                let bounds = context.bounds_in_parent(&children, Some(id))?.ok_or("Group has no geometry")?;
                let scale = frame.width / bounds.width;
                if (scale - frame.height / bounds.height).abs() > 1e-6 * scale.max(1.0) { return Err("Group scaling must be uniform".into()); }
                let linear = StructureTransform { x: 0.0, y: 0.0, rotation: geometry::normalize_rotation(frame.rotation), scale };
                let offset = geometry::compose(linear, StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
                movement::transform_group(context, draft, id, StructureTransform { x: frame.x - offset.x, y: frame.y - offset.y, ..linear })?;
            },
            StructureKind::Content => resize_content(context, draft, update)?,
        }
    }
    Ok(roots)
}
pub(super) fn arrange(context: &Context<'_>, draft: &mut Draft, updates: &[FrameUpdate]) -> Result<Vec<String>, String> {
    let ids = validate_updates(context, updates)?; let roots = context.roots(&ids, false)?; context.editable(&roots)?;
    let by_id: std::collections::HashMap<_,_> = updates.iter().map(|update| (update.node_id.as_str(), update)).collect();
    for id in &roots {
        let frame = &by_id[id.as_str()].frame;
        let before = oriented_origin(context, id)?;
        movement::move_root(context, draft, id, crate::spaces::canvas_state::contract::StructurePoint { x: frame.x - before.0, y: frame.y - before.1 })?;
    }
    Ok(roots)
}
fn resize_content(context: &Context<'_>, draft: &mut Draft, update: &FrameUpdate) -> Result<(), String> {
    let item = context.item(&update.node_id)?; let world = context.projection.world[&item.id];
    let frame = &update.frame;
    let parent = context.parent_world(item.parent_id.as_deref())?;
    let transform = geometry::compose(geometry::inverse(parent), StructureTransform { x: frame.x, y: frame.y, rotation: geometry::normalize_rotation(frame.rotation), ..world });
    let layout = draft.layouts.get_mut(&item.id).ok_or("Content layout is unavailable")?;
    if let StructureLayout::Content { transform: local, size, size_authority, appearance, .. } = layout {
        *local = transform; size.width = frame.width / world.scale; size.height = frame.height / world.scale;
        if let Some(authority) = &update.size_authority { *size_authority = authority.clone(); }
        if let Some(scale) = update.typography_scale {
            let mut value = appearance.as_ref().map(serde_json::to_value).transpose().map_err(|error| error.to_string())?.unwrap_or_else(|| serde_json::json!({}));
            value["typographyScale"] = serde_json::json!(scale);
            *appearance = Some(serde_json::from_value(value).map_err(|error| error.to_string())?);
        }
    }
    Ok(())
}
fn oriented_origin(context: &Context<'_>, id: &str) -> Result<(f64,f64), String> {
    let world = context.projection.world[id];
    if context.item(id)?.kind != StructureKind::Group { return Ok((world.x,world.y)); }
    let children = context.children.get(&Some(id)).cloned().unwrap_or_default();
    let bounds = context.bounds_in_parent(&children, Some(id))?.ok_or("Group has no geometry")?;
    let point = geometry::compose(world, StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
    Ok((point.x,point.y))
}
fn validate_updates(context: &Context<'_>, updates: &[FrameUpdate]) -> Result<Vec<String>, String> {
    if updates.is_empty() || updates.len() > 100_000 { return Err("A frame edit requires a bounded selection".into()); }
    let mut seen = std::collections::HashSet::new();
    for update in updates {
        if !seen.insert(&update.node_id) { return Err("A frame edit cannot update an item twice".into()); }
        context.item(&update.node_id)?;
        let frame = &update.frame;
        if ![frame.x,frame.y,frame.width,frame.height].into_iter().all(geometry::coordinate) || frame.width <= 0.0 || frame.height <= 0.0 || !frame.rotation.is_finite() { return Err("Invalid requested frame".into()); }
        if update.typography_scale.is_some_and(|scale| !scale.is_finite() || !(0.25..=4.0).contains(&scale)) { return Err("Invalid typography scale".into()); }
    }
    Ok(updates.iter().map(|update| update.node_id.clone()).collect())
}
