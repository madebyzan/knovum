use serde_json::Value;

use super::{
    scalars::{
        optional_string, optional_string_array, required_safe_nonnegative_integer, required_string,
    },
    validation::reject_unknown,
};

pub(crate) const MAX_DESCRIPTION: usize = 8_000;
pub(crate) const MAX_DETECTED_TEXT: usize = 16_000;
pub(crate) const MAX_TAGS: usize = 32;
pub(crate) const MAX_TAG: usize = 80;

pub(crate) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let object = value
        .as_object()
        .ok_or_else(|| "visualKnowledge must be an object".to_string())?;
    reject_unknown(
        object,
        [
            "sourceRevision",
            "description",
            "detectedText",
            "tags",
            "generatedAt",
            "executionSource",
        ],
        "visualKnowledge",
    )?;
    required_string(object, "sourceRevision", 128)?;
    required_string(object, "description", MAX_DESCRIPTION)?;
    optional_string(object, "detectedText", MAX_DETECTED_TEXT, false)?;
    if !object.contains_key("tags") {
        return Err("Space node data.tags must be an array".into());
    }
    optional_string_array(object, "tags", MAX_TAGS, MAX_TAG)?;
    required_safe_nonnegative_integer(object, "generatedAt")?;
    let source: crate::ai::tasks::ExecutionSource = serde_json::from_value(
        object
            .get("executionSource")
            .ok_or("Visual execution source is required")?
            .clone(),
    )
    .map_err(|e| format!("Invalid visual execution source: {e}"))?;
    source.validate()?;
    Ok(())
}
