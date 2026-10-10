mod fields;
mod image_comments;
mod image_depth;
mod image_edit;
mod resource;
mod tiptap;
mod website;

use serde_json::Value;

use super::{
    scalars::{optional_string, optional_string_array, required_string},
    validation::reject_unknown,
};

pub(super) const MAX_TEXT: usize = 4 * 1024 * 1024;
pub(super) const MAX_PREVIEW_TEXT: usize = 512 * 1024;
const COMMON_FIELDS: &[&str] = &[
    "kind",
    "title",
    "tags",
    "sourceRevision",
    "knowledgeRevision",
    "externalSource",
];

pub(super) fn validate_base_fields(kind: &str, payload: &Value) -> Result<(), String> {
    let object = payload
        .as_object()
        .ok_or_else(|| "Space node data must be an object".to_string())?;
    if required_string(object, "kind", 32)? != kind {
        return Err("Space node kind must equal data.kind".into());
    }
    required_string(object, "title", 512)?;
    optional_string_array(object, "tags", 64, 80)?;
    optional_string(object, "sourceRevision", 128, false)?;
    optional_string(object, "knowledgeRevision", 128, false)?;
    super::external_source::validate(object.get("externalSource"))?;
    Ok(())
}

pub(super) fn validate_payload_shape(kind: &str, payload: &Value) -> Result<(), String> {
    let object = payload
        .as_object()
        .ok_or_else(|| "Space node data must be an object".to_string())?;
    let specific = fields::specific_fields_for_kind(kind)?;
    let allowed = COMMON_FIELDS.iter().chain(specific).copied();
    reject_unknown(object, allowed, "Space node data")?;
    match kind {
        "note" | "text" => tiptap::validate_canvas_text(object),
        "markdown" => resource::validate_markdown(object),
        "website" => website::validate(object),
        "image" => {
            resource::validate_visual_asset(object)?;
            image_edit::validate(object.get("imageEdit"))?;
            image_depth::validate(object.get("imageDepth"))?;
            if object.contains_key("imageDepth") && object.contains_key("imageEdit") {
                return Err("Relative depth fields do not support photo edits".into());
            }
            image_comments::validate(object.get("comments"))?;
            super::visual::validate(object.get("visualKnowledge"))
        }
        "video" => {
            resource::validate_visual_asset(object)?;
            resource::validate_media_metadata(object)?;
            super::visual::validate(object.get("visualKnowledge"))?;
            super::media::validate(
                object.get("mediaKnowledge"),
                object.get("sourceRevision").and_then(Value::as_str),
            )
        }
        "audio" => {
            resource::validate_asset(object)?;
            resource::validate_media_metadata(object)?;
            super::media::validate(
                object.get("mediaKnowledge"),
                object.get("sourceRevision").and_then(Value::as_str),
            )
        }
        "file" => resource::validate_file(object),
        "code" => resource::validate_code(object),
        "credential" => resource::validate_credential(object),
        "object" | "portal" => Ok(()),
        _ => Err("Unsupported Space node kind".into()),
    }
}

#[cfg(test)]
mod tests;
