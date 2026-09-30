use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageDerivation {
    pub id: String,
    pub output_node_id: String,
    pub inputs: Vec<PackageDerivationInput>,
    pub action_kind: String,
    pub action_version: i64,
    pub engine: String,
    pub status: String,
    pub error: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: i64,
    pub grounded_document: Option<PackageGroundedDocument>,
    pub generated_image: Option<PackageGeneratedImage>,
    pub generated_video: Option<PackageGeneratedVideo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageGroundedDocument {
    pub citations: Value,
    pub execution_source: crate::ai::tasks::ExecutionSource,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageGeneratedImage {
    pub intent: String,
    pub style: String,
    pub aspect_ratio: String,
    pub execution_source: crate::ai::tasks::ExecutionSource,
    pub revised_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageGeneratedVideo {
    pub prompt: String,
    pub aspect_ratio: String,
    pub duration_seconds: i64,
    pub execution_source: crate::ai::tasks::ExecutionSource,
    pub operation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageDerivationInput {
    pub node_id: Option<String>,
    pub source_identity: String,
    pub source_snapshot: PackageSourceSnapshot,
    pub role: String,
    pub source_revision: String,
    pub revision_basis: String,
    pub input_revision: String,
    pub input_snapshot: Value,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSourceSnapshot {
    pub title: String,
    pub kind: String,
    pub original_url: Option<String>,
    pub platform: Option<String>,
}
