use serde_json::Value;

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
    let Some(source) = source_revision(data) else {
        return Ok(None);
    };
    if job_type != EnrichmentJobType::AnalyzeImage {
        return Ok(Some(source.to_owned()));
    }
    image_revision(data).map(Some)
}

pub(crate) fn image_revision(data: &Value) -> Result<String, String> {
    if data.get("kind").and_then(Value::as_str) != Some("image") {
        return Err("Image analysis requires an image node".into());
    }
    input_revision::resolve("image", data, RevisionBasis::Rendered).map(|(revision, _)| revision)
}
