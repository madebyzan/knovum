pub use crate::media::acquisition::contracts::MediaKind;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedMedia {
    pub kind: MediaKind,
    pub title: String,
    pub creator: Option<String>,
    pub duration_seconds: Option<u64>,
    pub intrinsic_width: Option<u32>,
    pub intrinsic_height: Option<u32>,
    pub asset_path: String,
    pub file_name: String,
    pub size: u64,
    pub modified_at: u64,
    pub source_revision: String,
    pub preview_path: Option<String>,
    pub origin: MediaOrigin,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaOrigin {
    pub kind: &'static str,
    pub url: String,
    pub provider: String,
    pub external_id: String,
}
