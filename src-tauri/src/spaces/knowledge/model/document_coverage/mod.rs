mod coverage;
mod validation;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) const MAX_RESOURCE_BYTES: u64 = 32 * 1024 * 1024;
pub(crate) const MAX_TOTAL_RESOURCE_BYTES: u64 = 256 * 1024 * 1024;
pub(crate) const MAX_RESOURCE_PIXELS: u64 = 24_000_000;
pub(crate) const MAX_RESOURCE_DIMENSION: u32 = 16_384;

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub(crate) enum DocumentParser {
    #[serde(rename = "pdf-inspector")]
    PdfInspector,
    #[serde(rename = "office-oxide")]
    OfficeOxide,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DocumentCoverage {
    pub schema_version: u8,
    pub parser: DocumentParser,
    pub source_revision: String,
    pub page_count: Option<usize>,
    pub pages_requiring_recognition: Vec<u32>,
    pub pages_with_extracted_text: Vec<u32>,
    pub pages_without_extractable_content: Vec<u32>,
    pub resources: Vec<DocumentResource>,
    pub occurrences: Vec<DocumentOccurrence>,
    pub warnings: Vec<String>,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DocumentResource {
    pub id: String,
    pub path: String,
    pub media_type: String,
    pub width: u32,
    pub height: u32,
    pub byte_size: u64,
}

/// One place the document shows a picture. The picture is either extracted as a durable raster,
/// so it can be viewed and saved on its own, or `unavailable` says why it could not be.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DocumentOccurrence {
    pub id: String,
    pub location: Value,
    pub context: String,
    pub alt_text: Option<String>,
    pub resource_id: Option<String>,
    pub pdf_input: Option<crate::documents::pdf_input::PdfInput>,
    pub unavailable: Option<String>,
}

impl DocumentCoverage {
    pub(crate) fn from_node_content(payload: &Value) -> Result<Self, String> {
        let report = Self::decode(
            payload
                .get("documentCoverage")
                .ok_or("Document coverage is missing")?,
        )?;
        if payload.get("sourceRevision").and_then(Value::as_str)
            != Some(report.source_revision.as_str())
        {
            return Err("Document coverage belongs to an obsolete source revision".into());
        }
        Ok(report)
    }
    pub(crate) fn decode(value: &Value) -> Result<Self, String> {
        let report: Self = serde_json::from_value(value.clone())
            .map_err(|error| format!("Invalid document coverage: {error}"))?;
        report.validate()?;
        Ok(report)
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        validation::validate(self)
    }
}
