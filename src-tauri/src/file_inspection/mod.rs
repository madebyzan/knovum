mod archive;
mod classification;
mod inspect;
mod revision;
mod text_preview;

pub use archive::archive_entries;

pub use classification::is_sensitive_file;
pub use classification::{classify_file, ResourceClassification};
pub use inspect::inspect_file;
pub(crate) use revision::content_revision_cancellable;
pub use revision::{content_revision, verify_content_revision};
use serde::{Deserialize, Serialize};
pub use text_preview::{modified_millis, read_html_preview, read_text_preview};

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../shared/contracts/generated/")]
pub struct ArchiveMember {
    pub path: String,
    pub kind: ArchiveMemberKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub size: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../shared/contracts/generated/")]
pub enum ArchiveMemberKind {
    File,
    Folder,
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
    pub entries: Option<Vec<ArchiveMember>>,
}

pub fn entry_search_text(entries: &[ArchiveMember]) -> String {
    entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
