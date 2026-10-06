use serde::{Deserialize, Serialize};

use crate::ai::models::ModelTargetSelection;

use super::super::video_config::VideoTranslationProvenance;

#[derive(Clone, Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct TranslateVideoDubbingRequest {
    pub job_id: String,
    pub project_id: String,
    pub expected_revision: i64,
    pub target_language: String,
    pub local_only: bool,
    pub execution_target: ModelTargetSelection,
    pub segments: Vec<VideoTranslationSegmentInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VideoTranslationSegmentInput {
    pub source_segment_id: String,
    pub source_segment_revision: u32,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VideoTranslationSegmentOutput {
    pub step_id: String,
    pub source_segment_id: String,
    pub source_segment_revision: u32,
    pub source_text: String,
    pub translated_text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VideoTranslation {
    pub provenance: VideoTranslationProvenance,
    pub segments: Vec<VideoTranslationSegmentOutput>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TranslationRunInput {
    pub workflow: String,
    pub project_id: String,
    pub revision_id: String,
    pub expected_revision: i64,
    pub input_revision: String,
    pub target_language: String,
    pub local_only: bool,
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub execution_target: ModelTargetSelection,
    pub segments: Vec<VideoTranslationSegmentInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TranslationStepConfiguration {
    pub project_id: String,
    pub revision_id: String,
    pub expected_revision: i64,
    pub input_revision: String,
    pub target_language: String,
    pub local_only: bool,
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub execution_target: ModelTargetSelection,
    pub ordinal: usize,
    pub segment: VideoTranslationSegmentInput,
}
