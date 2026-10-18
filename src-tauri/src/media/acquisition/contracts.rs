use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Audio,
    Video,
}

/// What a link turned out to be, reported once before the download starts.
///
/// The canvas reserves the node for an acquisition from this, so it has to be the same answer the
/// finished import publishes: the kind comes from the codecs of the selected format, and the asset
/// path from the one rule that names a downloaded media asset.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedMedia {
    pub kind: MediaKind,
    pub title: String,
    pub duration_seconds: Option<u64>,
    pub asset_path: String,
}

/// What a link turned out to be, as the downloader reads it off the selected format.
pub struct ResolvedSource {
    pub kind: MediaKind,
    pub title: String,
    pub duration_seconds: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquisitionProgress {
    pub stage: &'static str,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub processed_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

/// What an acquisition reports while it runs: first what the source is, then how far it has got.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum AcquisitionEvent {
    Resolved(ResolvedMedia),
    Progress(AcquisitionProgress),
}
