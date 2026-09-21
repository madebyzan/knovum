mod insights;
mod transcript;

use serde_json::Value;

pub(super) use super::scalars::required_safe_nonnegative_integer as nonnegative;
use super::{scalars::required_string, validation::reject_unknown};

pub(super) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let object = value
        .as_object()
        .ok_or_else(|| "mediaKnowledge must be an object".to_string())?;
    reject_unknown(
        object,
        ["transcript", "sourceRevision", "schemaVersion", "insights"],
        "mediaKnowledge",
    )?;
    if object.get("schemaVersion").and_then(Value::as_i64) != Some(4) {
        return Err("mediaKnowledge schemaVersion must be 4".into());
    }
    required_string(object, "sourceRevision", 128)?;
    let transcript = transcript::validate(object)?;
    if let Some(value) = object.get("insights") {
        insights::validate(value, &transcript)?;
    }
    Ok(())
}
