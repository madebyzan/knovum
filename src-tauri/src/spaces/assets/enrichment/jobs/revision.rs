use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::job::EnrichmentJobType;
use crate::spaces::derivation::input_revision::{self, RevisionBasis};

pub(super) fn source_revision(data: &Value) -> Option<&str> {
    if matches!(
        data.get("processingState").and_then(Value::as_str),
        Some("importing" | "processing")
    ) {
        return None;
    }
    data.get("sourceRevision")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

pub(super) fn expected_revision(
    data: &Value,
    job_type: EnrichmentJobType,
) -> Result<Option<String>, String> {
    if job_type == EnrichmentJobType::AnalyzeVisual {
        return visual_revision(data);
    }
    Ok(source_revision(data).map(str::to_owned))
}

/// Revision of the image actually sent to vision. Generated previews have immutable paths.
pub(crate) fn visual_revision(node_payload: &Value) -> Result<Option<String>, String> {
    let Some(source) = source_revision(node_payload) else {
        return Ok(None);
    };
    match node_payload.get("kind").and_then(Value::as_str) {
        Some("image") => input_revision::resolve("image", node_payload, RevisionBasis::Rendered)
            .map(|(revision, _)| Some(revision)),
        Some("video") => {
            let Some(preview) = node_payload
                .get("previewPath")
                .and_then(Value::as_str)
                .filter(|path| !path.trim().is_empty())
            else {
                return Ok(None);
            };
            let payload = serde_json::to_vec(&json!({
                "source": source,
                "preview": preview,
                "origin": node_payload.get("origin").and_then(|origin| origin.get("kind")),
            }))
            .map_err(|error| error.to_string())?;
            Ok(Some(format!(
                "visual1:{}",
                hex::encode(Sha256::digest(payload))
            )))
        }
        _ => Ok(None),
    }
}
