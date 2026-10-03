use serde::{Deserialize, Serialize};

fn empty_object() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PresetSynthesisConfig {
    pub speed: f32,
    pub sample_rate: u32,
}

pub(crate) fn parse_synthesis_config(
    value: &serde_json::Value,
) -> Result<PresetSynthesisConfig, String> {
    let config: PresetSynthesisConfig =
        serde_json::from_value(value.clone()).map_err(crate::spaces::db::json_error)?;
    if !config.speed.is_finite() || !(0.5..=2.0).contains(&config.speed) || config.sample_rate == 0
    {
        return Err("Voice synthesis configuration is invalid".to_owned());
    }
    Ok(config)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct VoiceProfileSummary {
    pub id: String,
    pub name: String,
    pub source_kind: String,
    pub active_revision_id: String,
    pub engine_id: String,
    pub model_spec_id: String,
    pub language: String,
    pub preset_voice_id: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpeechProjectRequest {
    pub space_id: String,
    pub project_type: String,
    pub title: String,
    #[serde(default = "empty_object")]
    pub output_config: serde_json::Value,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpeechProjectSummary {
    pub id: String,
    pub space_id: String,
    pub project_type: String,
    pub title: String,
    pub active_revision_id: String,
    pub active_revision: i64,
    pub segment_count: i64,
    pub generation_epoch: i64,
    pub active_generation_job_id: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechSegmentInput {
    pub id: String,
    pub chapter_key: Option<String>,
    pub speaker_key: Option<String>,
    pub text: String,
    pub voice_revision_id: String,
    pub language: String,
    pub target_start_ms: Option<i64>,
    pub target_end_ms: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpeechProjectRevision {
    pub id: String,
    pub project_id: String,
    pub revision: i64,
    pub source_revision: String,
    pub pronunciation_lexicon: String,
    pub output_config: String,
    pub output_asset_path: Option<String>,
    pub output_sha256: Option<String>,
    pub output_sample_rate: Option<i64>,
    pub output_sample_count: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpeechSegment {
    pub id: String,
    pub ordinal: i64,
    pub chapter_key: Option<String>,
    pub speaker_key: Option<String>,
    pub text: String,
    pub normalized_text: String,
    pub voice_revision_id: String,
    pub language: String,
    pub target_start_ms: Option<i64>,
    pub target_end_ms: Option<i64>,
    pub state: String,
    pub state_reason: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpeechProjectDetail {
    pub project: SpeechProjectSummary,
    pub revision: SpeechProjectRevision,
    pub segments: Vec<SpeechSegment>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechGenerationProgress {
    pub stage: String,
    pub completed: u32,
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources:
        Option<crate::local_resources::preparation::contracts::ResourcePreparationProgress>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewVoiceRequest {
    pub job_id: String,
    pub space_id: String,
    pub voice_revision_id: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechAudioOutput {
    pub path: String,
    pub sha256: String,
    pub sample_rate: u32,
    pub sample_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSpeechProjectRequest {
    pub job_id: String,
    pub project_id: String,
    pub expected_revision: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
    pub segment_id: String,
    pub path: String,
    pub sha256: String,
    pub sample_rate: i64,
    pub sample_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechProjectRenderOutput {
    pub project_id: String,
    pub revision: i64,
    pub renders: Vec<SpeechSegmentRenderOutput>,
    pub output: SpeechAudioOutput,
}

#[derive(Debug, sqlx::FromRow)]
pub(super) struct PresetVoiceExecution {
    pub revision_id: String,
    pub engine_id: String,
    pub model_spec_id: String,
    pub language: String,
    pub preset_voice_id: String,
    pub synthesis_config: String,
}

#[derive(Debug, sqlx::FromRow)]
pub(super) struct SpeechSegmentExecution {
    pub id: String,
    pub normalized_text: String,
    pub voice_revision_id: String,
    pub language: String,
}
