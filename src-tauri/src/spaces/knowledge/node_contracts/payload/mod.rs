mod fields;
mod resource;
mod subtitle;
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
];

pub(super) fn validate_base_fields(kind: &str, data: &Value) -> Result<(), String> {
    let object = data
        .as_object()
        .ok_or_else(|| "Space node data must be an object".to_string())?;
    if required_string(object, "kind", 32)? != kind {
        return Err("Space node kind must equal data.kind".into());
    }
    required_string(object, "title", 512)?;
    optional_string_array(object, "tags", 64, 80)?;
    optional_string(object, "sourceRevision", 128, false)?;
    optional_string(object, "knowledgeRevision", 128, false)?;
    Ok(())
}

pub(super) fn validate_payload_shape(kind: &str, data: &Value) -> Result<(), String> {
    let object = data
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
            super::visual::validate(object.get("visualKnowledge"))
        }
        "video" => {
            resource::validate_visual_asset(object)?;
            resource::validate_media_metadata(object)?;
            super::visual::validate(object.get("visualKnowledge"))?;
            super::media::validate(object.get("mediaKnowledge"))
        }
        "audio" => {
            resource::validate_asset(object)?;
            resource::validate_media_metadata(object)?;
            super::media::validate(object.get("mediaKnowledge"))
        }
        "subtitle" => subtitle::validate(object),
        "file" => resource::validate_file(object),
        "code" => resource::validate_code(object),
        "credential" => resource::validate_credential(object),
        "object" | "portal" => Ok(()),
        _ => Err("Unsupported Space node kind".into()),
    }
}


#[cfg(test)]
mod tests {
    use super::validate_payload_shape;
    use serde_json::json;

    #[test]
    fn directory_and_archive_node_kinds_are_not_space_contracts() {
        for kind in ["folder", "archive"] {
            let data = json!({"kind": kind, "title": "Project", "fileName": "Project", "assetPath": "/assets/project"});
            assert_eq!(
                validate_payload_shape(kind, &data).unwrap_err(),
                "Unsupported Space node kind"
            );
        }
    }
}
