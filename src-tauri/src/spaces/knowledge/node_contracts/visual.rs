use serde_json::Value;

use super::{
    scalars::{
        optional_string, optional_string_array, required_safe_nonnegative_integer, required_string,
    },
    validation::reject_unknown,
};

pub(super) fn validate(value: Option<&Value>) -> Result<(), String> {
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
            "provider",
            "model",
        ],
        "visualKnowledge",
    )?;
    required_string(object, "sourceRevision", 128)?;
    required_string(object, "description", 8_000)?;
    optional_string(object, "detectedText", 16_000, false)?;
    if !object.contains_key("tags") {
        return Err("Space node data.tags must be an array".into());
    }
    optional_string_array(object, "tags", 32, 80)?;
    required_safe_nonnegative_integer(object, "generatedAt")?;
    required_string(object, "provider", 128)?;
    required_string(object, "model", 256)?;
    Ok(())
}
