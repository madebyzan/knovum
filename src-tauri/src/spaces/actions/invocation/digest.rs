use serde_json::Value;
use sha2::{Digest, Sha256};

use super::ActionInvocationRequest;
use crate::spaces::db;

pub(super) fn request_digest(request: &ActionInvocationRequest) -> Result<String, String> {
    let value = serde_json::json!({
        "spaceId": request.space_id,
        "actionId": request.action_id,
        "actionVersion": request.action_version,
        "parameters": request.parameters,
        "inputSnapshot": request.input_snapshot,
        "expectedRevisions": request.expected_revisions,
    });
    let encoded = serde_json::to_vec(&canonicalize(value)).map_err(db::json_error)?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

pub(super) fn canonicalize(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, canonicalize(value)))
                .collect(),
        ),
        scalar => scalar,
    }
}
