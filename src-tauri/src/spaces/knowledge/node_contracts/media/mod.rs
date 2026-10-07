pub(super) mod analysis;
mod insights;
mod transcript;

#[cfg(test)]
mod tests;

use serde_json::Value;

pub(super) use super::scalars::required_safe_nonnegative_integer as nonnegative;
use super::{scalars::required_string, validation::reject_unknown};

pub(super) fn validate(
    value: Option<&Value>,
    node_source_revision: Option<&str>,
) -> Result<(), String> {
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
    if object.get("schemaVersion").and_then(Value::as_i64) != Some(1) {
        return Err("mediaKnowledge schemaVersion must be 1".into());
    }
    let source_revision = required_string(object, "sourceRevision", 128)?;
    if Some(source_revision) != node_source_revision {
        return Err("mediaKnowledge sourceRevision must match node sourceRevision".into());
    }
    let transcript = transcript::validate(object)?;
    if let Some(value) = object.get("insights") {
        insights::validate(value, &transcript)?;
    }
    Ok(())
}
