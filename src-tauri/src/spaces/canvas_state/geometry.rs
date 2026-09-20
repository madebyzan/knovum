use std::collections::{HashMap, VecDeque};
use super::contract::{SpaceEditorDocument, StructureKind, StructureLayout, StructureTransform, StructureSize};
pub(in crate::spaces) const COORDINATE_LIMIT: f64 = 1_000_000_000_000.0;
pub(in crate::spaces) const SCALE_MIN: f64 = 0.000001;
pub(in crate::spaces) const SCALE_MAX: f64 = 1_000_000.0;
pub(in crate::spaces) fn identity() -> StructureTransform { StructureTransform { x: 0.0, y: 0.0, rotation: 0.0, scale: 1.0 } }
pub(in crate::spaces) fn normalize_rotation(value: f64) -> f64 { (value + 180.0).rem_euclid(360.0) - 180.0 }
pub(in crate::spaces) fn compose(parent: StructureTransform, local: StructureTransform) -> StructureTransform {
    let (sin, cos) = parent.rotation.to_radians().sin_cos();
    StructureTransform { x: parent.x + parent.scale * (cos * local.x - sin * local.y), y: parent.y + parent.scale * (sin * local.x + cos * local.y),
        rotation: normalize_rotation(parent.rotation + local.rotation), scale: parent.scale * local.scale }
}
pub(in crate::spaces) fn inverse(value: StructureTransform) -> StructureTransform {
    let inverse = StructureTransform { x: 0.0, y: 0.0, rotation: -value.rotation, scale: 1.0 / value.scale };
    let translated = compose(inverse, StructureTransform { x: -value.x, y: -value.y, ..identity() });
    StructureTransform { x: translated.x, y: translated.y, ..inverse }
}
pub(in crate::spaces) fn coordinate(value: f64) -> bool { value.is_finite() && value.abs() <= COORDINATE_LIMIT }
pub(in crate::spaces) fn valid_transform(value: StructureTransform) -> bool {
    coordinate(value.x) && coordinate(value.y) && value.rotation.is_finite() && (-180.0..180.0).contains(&value.rotation)
        && value.scale.is_finite() && (SCALE_MIN..=SCALE_MAX).contains(&value.scale)
}
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::spaces) struct Bounds { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }
pub(in crate::spaces) fn transformed_bounds(transform: StructureTransform, size: StructureSize) -> Bounds {
    let points = [(0.0, 0.0), (size.width, 0.0), (0.0, size.height), (size.width, size.height)].map(|(x,y)| compose(transform, StructureTransform { x, y, ..identity() }));
    let x = points.iter().map(|point| point.x).fold(f64::INFINITY, f64::min);
    let y = points.iter().map(|point| point.y).fold(f64::INFINITY, f64::min);
    let right = points.iter().map(|point| point.x).fold(f64::NEG_INFINITY, f64::max);
    let bottom = points.iter().map(|point| point.y).fold(f64::NEG_INFINITY, f64::max);
    Bounds { x, y, width: right - x, height: bottom - y }
}
pub(in crate::spaces) struct Projection { pub world: HashMap<String, StructureTransform>, pub bounds: HashMap<String, Bounds>, pub order: Vec<String> }
/** Input topology is validated before projection; no engine owns this geometry. */
pub(in crate::spaces) fn project(document: &SpaceEditorDocument) -> Result<Projection, String> {
    let items: HashMap<_,_> = document.structure.items.iter().map(|item| (item.id.as_str(), item)).collect();
    let layouts: HashMap<_,_> = document.layout.items.iter().map(|layout| (layout.id(), layout)).collect();
    let mut children: HashMap<Option<&str>, Vec<&str>> = HashMap::new();
    for item in &document.structure.items { children.entry(item.parent_id.as_deref()).or_default().push(item.id.as_str()); }
    for siblings in children.values_mut() { siblings.sort_by_key(|id| items[id].order); }
    let mut pending: VecDeque<&str> = children.get(&None).into_iter().flatten().copied().collect();
    let mut world = HashMap::new(); let mut bounds = HashMap::new(); let mut order = Vec::new();
    while let Some(id) = pending.pop_front() {
        let item = items[id]; let layout = layouts.get(id).ok_or("Structure item has no layout")?;
        let parent = item.parent_id.as_deref().map(|id| world.get(id).copied().ok_or("Unreachable parent")).transpose()?.unwrap_or_else(identity);
        let transform = compose(parent, layout.local());
        if !valid_transform(transform) { return Err("Composed world transform exceeds the canonical limits".into()); }
        if let StructureLayout::Content { size, .. } | StructureLayout::Section { size, .. } = layout {
            let rect = transformed_bounds(transform, *size);
            if rect.width <= 0.0 || rect.height <= 0.0 || ![rect.x,rect.y,rect.x+rect.width,rect.y+rect.height].into_iter().all(coordinate) { return Err("World geometry is not representable".into()); }
            bounds.insert(id.to_string(), rect);
        }
        world.insert(id.to_string(), transform); order.push(id.to_string());
        if let Some(children) = children.get(&Some(id)) { for child in children.iter().rev() { pending.push_front(child); } }
    }
    if order.len() != items.len() { return Err("Structure contains an unreachable item".into()); }
    for id in order.iter().rev() {
        if items[id.as_str()].kind != StructureKind::Group { continue; }
        let rectangles: Vec<_> = children.get(&Some(id.as_str())).into_iter().flatten().map(|child| bounds.get(*child).copied().ok_or("Group child has no bounds")).collect::<Result<_,_>>()?;
        bounds.insert(id.clone(), union_bounds(&rectangles).ok_or("Empty Group")?);
    }
    Ok(Projection { world, bounds, order })
}
pub(in crate::spaces) fn union_bounds(rectangles: &[Bounds]) -> Option<Bounds> {
    if rectangles.is_empty() { return None; }
    let x = rectangles.iter().map(|rect| rect.x).fold(f64::INFINITY, f64::min);
    let y = rectangles.iter().map(|rect| rect.y).fold(f64::INFINITY, f64::min);
    let right = rectangles.iter().map(|rect| rect.x + rect.width).fold(f64::NEG_INFINITY, f64::max);
    let bottom = rectangles.iter().map(|rect| rect.y + rect.height).fold(f64::NEG_INFINITY, f64::max);
    Some(Bounds { x, y, width: right - x, height: bottom - y })
}
