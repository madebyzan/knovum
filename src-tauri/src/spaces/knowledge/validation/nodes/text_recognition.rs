use serde_json::Value;

use super::{
    scalars::{optional_string, required_safe_nonnegative_integer, required_string},
    validation::reject_unknown,
};

/// What a node keeps of its recognized text. The whole text lives in its knowledge chunks.
pub(crate) const MAX_TEXT: usize = 16_000;

pub(crate) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let object = value
        .as_object()
        .ok_or_else(|| "textRecognition must be an object".to_string())?;
    reject_unknown(
        object,
        ["sourceRevision", "modelSpecId", "generatedAt", "text"],
        "textRecognition",
    )?;
    required_string(object, "sourceRevision", 128)?;
    required_string(object, "modelSpecId", 160)?;
    required_safe_nonnegative_integer(object, "generatedAt")?;
    optional_string(object, "text", MAX_TEXT, false)?;
    Ok(())
}
