use serde::{Deserialize, Serialize};

use super::super::contracts::SpeechAudioOutput;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum VoiceCandidateSource {
    Reference {
        source_path: String,
        source_revision: String,
        start_ms: u64,
        end_ms: u64,
        reference_text: Option<String>,
        reference_text_confirmed: bool,
    },
    Designed {
        description: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct DraftVoiceCandidateRequest {
    pub request_id: String,
    pub space_id: String,
    pub language: String,
    pub text: String,
    pub seed: u32,
    pub source: VoiceCandidateSource,
    pub authorized: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum VoiceCandidateStatus {
    Ready,
    Selected,
    Rejected,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VoiceCandidate {
    pub candidate_id: String,
    pub run_id: String,
    pub space_id: String,
    pub source_kind: String,
    pub language: String,
    pub sample_text: String,
    pub seed: u32,
    pub design_description: Option<String>,
    pub reference_text: Option<String>,
    pub audio: SpeechAudioOutput,
    pub status: VoiceCandidateStatus,
    pub auditioned_at: Option<i64>,
    pub selected_voice_profile_id: Option<String>,
    pub selected_voice_revision_id: Option<String>,
    pub selected_at: Option<i64>,
    pub rejected_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CreateCandidateVoiceRequest {
    pub candidate_id: String,
    pub space_id: String,
    pub name: String,
    pub authorized: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct VoiceCandidateStepConfiguration {
    pub request: DraftVoiceCandidateRequest,
    pub reference_source: Option<ReferenceSourceSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ReferenceSourceSnapshot {
    pub asset_path: String,
    pub byte_length: u64,
    pub source_revision: String,
}
