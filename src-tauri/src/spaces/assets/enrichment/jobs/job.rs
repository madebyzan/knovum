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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum EnrichmentJobType {
    AnalyzeVisual,
    AnalyzeMedia,
    TranscribeMedia,
    ParseDocument,
    AnalyzeDocument,
}

impl EnrichmentJobType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::AnalyzeVisual => "analyze_visual",
            Self::AnalyzeMedia => "analyze_media",
            Self::TranscribeMedia => "transcribe_media",
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum EnrichmentJobStatus {
    Pending,
    Running,
    AwaitingConfiguration,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentJobState {
    pub id: String,
    pub space_id: String,
    pub node_id: String,
    pub job_type: EnrichmentJobType,
    pub status: EnrichmentJobStatus,
    pub source_revision: String,
    pub attempts: i64,
    pub execution_target: Option<crate::ai::tasks::TaskTargetSelection>,
    pub error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentJobProjection {
    pub job: EnrichmentJobState,
    pub space_title: String,
    pub node_title: String,
    pub node_kind: String,
}
