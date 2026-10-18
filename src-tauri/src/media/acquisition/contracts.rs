use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Audio,
    Video,
}

/// How far an acquisition has got. The kind of media is not reported here: it is what the link
/// asked for, so the canvas already holds the node the arriving bytes belong to.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquisitionProgress {
    pub stage: &'static str,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub processed_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}
