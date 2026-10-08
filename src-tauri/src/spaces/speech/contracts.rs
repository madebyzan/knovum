use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub(super) struct ProjectRevisionIdentity<'a> {
    pub project_id: &'a str,
    pub revision_id: &'a str,
}

fn empty_object() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct VoiceSynthesisConfig {
    pub speed: f32,
    pub sample_rate: u32,
    pub seed: u32,
}

pub(crate) fn parse_synthesis_config(
    value: &serde_json::Value,
) -> Result<VoiceSynthesisConfig, String> {
    let config = VoiceSynthesisConfig::deserialize(value).map_err(crate::spaces::db::json_error)?;
    if !config.speed.is_finite() || !(0.5..=2.0).contains(&config.speed) || config.sample_rate == 0
    {
        return Err("Voice synthesis configuration is invalid".to_owned());
    }
    Ok(config)
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CreatePresetVoiceRequest {
    pub space_id: String,
    pub name: String,
    pub preset_voice_id: String,
    pub engine_id: String,
    pub model_spec_id: String,
    pub language: String,
    #[serde(default = "empty_object")]
    pub synthesis_config: serde_json::Value,
}

#[derive(Debug, Serialize, sqlx::FromRow, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VoiceLibraryRevision {
    pub profile_id: String,
    pub name: String,
    pub source_kind: String,
    pub revision_id: String,
    pub revision: i64,
    pub is_active: bool,
    pub supports_delivery_instruction: bool,
    pub engine_id: String,
    pub model_spec_id: String,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preset_voice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub archived_at: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CreateSpeechProjectRequest {
    pub space_id: String,
    pub source_node_id: String,
    pub project_type: String,
    pub title: String,
    #[serde(default = "empty_object")]
    pub output_config: serde_json::Value,
}

#[derive(Debug, Serialize, sqlx::FromRow, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechProjectOverview {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub project_type: String,
    pub title: String,
    pub active_revision_id: String,
    pub active_revision: i64,
    pub segment_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub active_run_id: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SaveSpeechProjectRevisionRequest {
    pub project_id: String,
    pub expected_revision: i64,
    pub source_revision: String,
    #[serde(default = "empty_object")]
    pub pronunciation_lexicon: serde_json::Value,
    #[serde(default = "empty_object")]
    pub output_config: serde_json::Value,
    pub segments: Vec<SpeechSegmentInput>,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechSegmentInput {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chapter_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub speaker_key: Option<String>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub delivery_instruction: Option<String>,
    pub voice_revision_id: String,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_start_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_end_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_segment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_segment_revision: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_text: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechProjectRevision {
    pub id: String,
    pub project_id: String,
    pub revision: i64,
    pub source_revision: String,
    #[sqlx(json)]
    pub pronunciation_lexicon: std::collections::BTreeMap<String, String>,
    #[sqlx(json)]
    pub output_config: super::project_config::ProjectOutputConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub output_asset_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub output_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub output_sample_rate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub output_sample_count: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechSegment {
    pub id: String,
    pub ordinal: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chapter_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub speaker_key: Option<String>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub delivery_instruction: Option<String>,
    pub normalized_text: String,
    pub voice_revision_id: String,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_start_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_end_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_segment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_segment_revision: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_text: Option<String>,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub state_reason: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechProjectSnapshot {
    pub project: SpeechProjectOverview,
    pub revision: SpeechProjectRevision,
    pub segments: Vec<SpeechSegment>,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechGenerationProgress {
    pub stage: String,
    pub completed: u32,
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub resources:
        Option<crate::local_resources::preparation::contracts::ResourcePreparationProgress>,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechPreviewAudioChunk {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub seq: u64,
    pub sample_rate: u32,
    pub sample_count: u64,
    pub pcm16: Vec<u8>,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct PreviewVoiceRequest {
    pub job_id: String,
    pub space_id: String,
    pub source: VoicePreviewSource,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub delivery_instruction: Option<String>,
    pub pronunciation_lexicon: serde_json::Value,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum VoicePreviewSource {
    Revision {
        #[serde(rename = "revisionId")]
        revision_id: String,
    },
    Preset {
        #[serde(rename = "speakerId")]
        speaker_id: String,
        language: String,
    },
}

pub use super::voice_candidates::{
    CreateCandidateVoiceRequest, DraftVoiceCandidateRequest, VoiceCandidate, VoiceCandidateStatus,
};

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechAudioOutput {
    pub path: String,
    pub sha256: String,
    pub sample_rate: u32,
    pub sample_count: u64,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct GenerateSpeechProjectRequest {
    pub request_id: String,
    pub project_id: String,
    pub expected_revision: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechSegmentRenderOutput {
    pub segment_id: String,
    pub path: String,
    pub sha256: String,
    pub sample_rate: u32,
    pub sample_count: u64,
    pub cached: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub(super) struct CachedSpeechRender {
    pub id: String,
    pub path: String,
    pub sha256: String,
    pub sample_rate: i64,
    pub sample_count: i64,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechProjectRenderOutput {
    pub project_id: String,
    pub revision: i64,
    pub renders: Vec<SpeechSegmentRenderOutput>,
    pub output: SpeechAudioOutput,
}

#[derive(Debug, sqlx::FromRow)]
pub(super) struct VoiceExecution {
    pub revision_id: String,
    pub source_kind: String,
    pub engine_id: String,
    pub model_spec_id: String,
    pub language: String,
    pub preset_voice_id: Option<String>,
    pub reference_asset_id: Option<String>,
    pub reference_source_revision: Option<String>,
    pub reference_text: Option<String>,
    pub design_description: Option<String>,
    pub synthesis_config: String,
}

#[derive(Debug, sqlx::FromRow)]
pub(super) struct SpeechSegmentExecution {
    pub id: String,
    pub normalized_text: String,
    pub delivery_instruction: Option<String>,
    pub voice_revision_id: String,
    pub language: String,
}
