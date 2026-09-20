use std::collections::BTreeMap;
use serde_json::Value;
use super::{context::Context, draft::Draft, editor_command::AppearanceField};
use crate::spaces::canvas_state::contract::StructureLayout;
pub(super) fn apply(context: &Context<'_>, draft: &mut Draft, id: &str, patch: &BTreeMap<AppearanceField, Option<Value>>) -> Result<(), String> {
    context.editable(&[id.into()])?;
    let layout = draft.layouts.get_mut(id).ok_or("Appearance target is unavailable")?;
    let appearance = match layout { StructureLayout::Content { appearance, .. } | StructureLayout::Section { appearance, .. } => appearance,
        StructureLayout::Group { .. } => return Err("Groups have no independent appearance".into()) };
    let mut value = appearance.as_ref().map(serde_json::to_value).transpose().map_err(|error| error.to_string())?.unwrap_or_else(|| serde_json::json!({}));
    let object = value.as_object_mut().ok_or("Invalid current appearance")?;
    for (field,value) in patch {
        let key = serde_json::to_value(field).map_err(|error| error.to_string())?;
        let key = key.as_str().ok_or("Invalid appearance field")?;
        match value { Some(value) => { object.insert(key.to_string(), value.clone()); }, None => { object.remove(key); } }
    }
    if object.is_empty() { *appearance = None; }
    else { *appearance = Some(serde_json::from_value(value).map_err(|error| error.to_string())?); }
    Ok(())
}
