use serde_json::Value;

use super::{
    scalars::{required_safe_nonnegative_integer, required_string},
    validation::reject_unknown,
};

/// That a node's pictures were read, and from which input. The text itself lives only in the
/// node's passages, so a document read page by page never grows the record every node read pays for.
pub(crate) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let object = value
        .as_object()
        .ok_or_else(|| "textRecognition must be an object".to_string())?;
    reject_unknown(
        object,
        ["sourceRevision", "modelSpecId", "generatedAt"],
        "textRecognition",
    )?;
    required_string(object, "sourceRevision", 128)?;
    required_string(object, "modelSpecId", 160)?;
    required_safe_nonnegative_integer(object, "generatedAt")?;
    Ok(())
}
