mod archive;
mod classification;
mod inspect;
mod revision;
mod text_preview;

pub use archive::archive_entries;
pub use classification::{classify_file, is_sensitive_file, ResourceClassification};
pub use inspect::inspect_file;
pub use revision::content_revision;
use serde::{Deserialize, Serialize};
pub use text_preview::{modified_millis, read_html_preview, read_text_preview};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedEntry {
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedFile {
    pub name: String,
    pub path: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_language: Option<String>,
    pub size: u64,
    pub modified_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<InspectedEntry>>,
}

pub fn entry_search_text(entries: &[InspectedEntry]) -> String {
    entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
