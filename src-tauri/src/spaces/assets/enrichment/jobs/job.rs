use serde::{Deserialize, Serialize};

pub(crate) const STATUS_PENDING: &str = "pending";
pub(crate) const STATUS_RUNNING: &str = "running";
pub(crate) const STATUS_SUCCEEDED: &str = "succeeded";
pub(crate) const STATUS_FAILED: &str = "failed";
pub(crate) const STATUS_CANCELLED: &str = "cancelled";

/// Prior statuses each transition may replace. A cancelled job cannot be rewritten by the
/// execution that returns afterwards.
pub(crate) const FROM_RUNNING: &str = "'running'";
pub(crate) const FROM_LIVE: &str = "'pending','running','awaiting_configuration'";

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnrichmentJobType {
    EmbedText,
    AnalyzeImage,
    AnalyzeMedia,
    TranscribeMedia,
    AnalyzeVideoCover,
    EmbedMediaTranscript,
    ParseDocument,
    AnalyzeDocument,
}

impl EnrichmentJobType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::EmbedText => "embed_text",
            Self::AnalyzeImage => "analyze_image",
            Self::AnalyzeMedia => "analyze_media",
            Self::TranscribeMedia => "transcribe_media",
            Self::AnalyzeVideoCover => "analyze_video_cover",
            Self::EmbedMediaTranscript => "embed_media_transcript",
            Self::ParseDocument => "parse_document",
            Self::AnalyzeDocument => "analyze_document",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueEnrichmentJobRequest {
    pub space_id: String,
    pub node_id: String,
    pub job_type: EnrichmentJobType,
    pub source_revision: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentJobRecord {
    pub id: String,
    pub space_id: String,
    pub node_id: String,
    pub job_type: String,
    pub status: String,
    pub source_revision: String,
    pub attempts: i64,
    pub error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
