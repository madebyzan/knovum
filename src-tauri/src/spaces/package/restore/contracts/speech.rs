use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageVoiceProfile {
    pub id: String,
    pub name: String,
    pub source_kind: String,
    pub active_revision_id: Option<String>,
    pub archived_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageVoiceRevision {
    pub id: String,
    pub profile_id: String,
    pub revision: i64,
    pub engine_id: String,
    pub model_spec_id: String,
    pub language: String,
    pub preset_voice_id: Option<String>,
    pub reference_asset_id: Option<String>,
    pub reference_source_revision: Option<String>,
    pub reference_text: Option<String>,
    pub design_description: Option<String>,
    pub synthesis_config: Value,
    pub authorization: Value,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageVoiceCandidate {
    pub id: String,
    pub run_id: String,
    pub source_kind: String,
    pub language: String,
    pub sample_text: String,
    pub design_description: Option<String>,
    pub reference_text: Option<String>,
    pub input_source_revision: Option<String>,
    pub input_byte_length: Option<i64>,
    pub reference_asset_path: String,
    pub reference_sha256: String,
    pub reference_byte_length: i64,
    pub preview_asset_path: String,
    pub preview_sha256: String,
    pub preview_sample_rate: i64,
    pub preview_sample_count: i64,
    pub model_spec_id: String,
    pub seed: i64,
    pub synthesis_config: Value,
    pub authorization: Value,
    pub status: String,
    pub auditioned_at: Option<i64>,
    pub selected_voice_profile_id: Option<String>,
    pub selected_voice_revision_id: Option<String>,
    pub selected_at: Option<i64>,
    pub rejected_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpeechSourceSeparation {
    pub id: String,
    pub run_id: String,
    pub source_asset_path: String,
    pub source_revision: String,
    pub source_duration_ms: i64,
    pub model_spec_id: String,
    pub background_asset_path: String,
    pub background_sha256: String,
    pub background_sample_rate: i64,
    pub background_sample_count: i64,
    pub background_duration_ms: i64,
    pub review_status: String,
    pub review_warning: String,
    pub created_at: i64,
    pub reviewed_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpeechProject {
    pub id: String,
    pub project_type: String,
    pub title: String,
    pub active_revision_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpeechProjectRevision {
    pub id: String,
    pub project_id: String,
    pub revision: i64,
    pub source_revision: String,
    pub pronunciation_lexicon: Value,
    pub output_config: Value,
    pub output_asset_path: Option<String>,
    pub output_sha256: Option<String>,
    pub output_sample_rate: Option<i64>,
    pub output_sample_count: Option<i64>,
    pub output_published_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpeechSegment {
    pub id: String,
    pub project_revision_id: String,
    pub ordinal: i64,
    pub chapter_key: Option<String>,
    pub speaker_key: Option<String>,
    pub text: String,
    pub normalized_text: String,
    pub voice_revision_id: String,
    pub language: String,
    pub target_start_ms: Option<i64>,
    pub target_end_ms: Option<i64>,
    pub source_segment_id: Option<String>,
    pub source_segment_revision: Option<i64>,
    pub source_text: Option<String>,
    pub state: String,
    pub state_reason: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpeechRender {
    pub id: String,
    pub segment_id: String,
    pub attempt_epoch: i64,
    pub input_digest: String,
    pub model_spec_id: String,
    pub engine_version: String,
    pub seed: Option<i64>,
    pub synthesis_config: Value,
    pub state: String,
    pub asset_path: Option<String>,
    pub output_sha256: Option<String>,
    pub sample_rate: Option<i64>,
    pub sample_count: Option<i64>,
    pub created_at: i64,
    pub published_at: Option<i64>,
}
