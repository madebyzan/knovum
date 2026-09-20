use std::collections::{HashMap, HashSet};
use super::contract::{CanvasNodeAppearance, SpaceEditorDocument, StructureLayout, StructureSize};
use super::geometry::{coordinate, valid_transform, COORDINATE_LIMIT};
pub(super) fn validate(document: &SpaceEditorDocument) -> Result<(), String> {
    if document.layout.version != 1 || document.layout.items.len() != document.structure.items.len() { return Err("Structure and layout must have exactly matching identities".into()); }
    let kinds: HashMap<_,_> = document.structure.items.iter().map(|item| (item.id.as_str(), item.kind)).collect();
    let mut seen = HashSet::new();
    for layout in &document.layout.items {
        if !seen.insert(layout.id()) || kinds.get(layout.id()) != Some(&layout.kind()) { return Err("Duplicate, orphan or mismatched layout identity".into()); }
        if !valid_transform(layout.local()) { return Err("Invalid local transform".into()); }
        match layout {
            StructureLayout::Content { size, file_subpath, .. } => {
                validate_size(*size)?;
                if file_subpath.as_deref().is_some_and(|path| !path.starts_with('#')) { return Err("fileSubpath must start with #".into()); }
            },
            StructureLayout::Section { size, position, .. } => {
                validate_size(*size)?;
                if !coordinate(position.x) || !coordinate(position.y) { return Err("Invalid Section position".into()); }
            },
            StructureLayout::Group { .. } => {},
        }
        if let Some(appearance) = layout.appearance() { validate_appearance(appearance)?; }
    }
    super::geometry::project(document)?;
    Ok(())
}
fn validate_size(size: StructureSize) -> Result<(), String> {
    if [size.width,size.height].into_iter().any(|value| !value.is_finite() || value <= 0.0 || value > COORDINATE_LIMIT) { return Err("Invalid layout size".into()); }
    Ok(())
}
pub(in crate::spaces) fn validate_appearance(value: &CanvasNodeAppearance) -> Result<(), String> {
    if value.fill.is_some() && value.custom_fill.is_some() { return Err("fill and customFill are mutually exclusive".into()); }
    if value.custom_fill.as_deref().is_some_and(|color| color.len() != 7 || !color.starts_with('#') || !color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)) { return Err("Custom fill must be #RRGGBB".into()); }
    if value.font_size.is_some_and(|number| !number.is_finite() || !(2.5..=576.0).contains(&number)) { return Err("Invalid font size".into()); }
    if value.typography_scale.is_some_and(|number| !number.is_finite() || !(0.25..=4.0).contains(&number)) { return Err("Invalid typography scale".into()); }
    Ok(())
}
