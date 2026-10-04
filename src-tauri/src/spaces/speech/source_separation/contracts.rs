use serde::{Deserialize, Serialize};

use super::super::video_config::SpeechMediaSource;

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct StartSpeechSourceSeparationRequest {
    pub request_id: String,
    pub space_id: String,
    pub source: SpeechMediaSource,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ReviewSpeechSourceSeparationRequest {
    pub separation_id: String,
    pub expected_review_status: SpeechSourceSeparationReviewStatus,
    pub accepted: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpeechSourceSeparationReviewStatus {
    Pending,
    Accepted,
    Rejected,
}

impl SpeechSourceSeparationReviewStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }

    pub(super) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            _ => Err("Source separation review status is invalid".into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechSourceSeparationCandidate {
    pub separation_id: String,
    pub space_id: String,
    pub run_id: String,
    pub source: SpeechMediaSource,
    pub background: SpeechMediaSource,
    pub model_spec_id: String,
    pub review_status: SpeechSourceSeparationReviewStatus,
    pub review_warning: String,
}
