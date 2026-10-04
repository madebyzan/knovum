use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct RenameVoiceProfileRequest {
    pub profile_id: String,
    pub expected_updated_at: i64,
    pub name: String,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ReviseVoiceProfileRequest {
    pub profile_id: String,
    pub expected_active_revision_id: String,
    pub language: String,
    pub speed: f32,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SetVoiceProfileArchivedRequest {
    pub profile_id: String,
    pub expected_updated_at: i64,
    pub archived: bool,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct DeleteVoiceProfileRequest {
    pub profile_id: String,
    pub expected_updated_at: i64,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CopyVoiceProfileRequest {
    pub profile_id: String,
    pub expected_active_revision_id: String,
    pub target_space_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VoiceRevisionSummary {
    pub id: String,
    pub revision: i64,
    pub engine_id: String,
    pub model_spec_id: String,
    pub language: String,
    pub speed: f32,
    pub sample_rate: u32,
    pub reference_source_revision: Option<String>,
    pub design_description: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VoiceProfileDetail {
    pub id: String,
    pub space_id: String,
    pub name: String,
    pub source_kind: String,
    pub active_revision_id: String,
    pub archived_at: Option<i64>,
    pub updated_at: i64,
    pub referenced_segment_count: i64,
    pub revisions: Vec<VoiceRevisionSummary>,
}
