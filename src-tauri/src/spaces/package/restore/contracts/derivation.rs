use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageDerivationRow {
    pub id: String,
    pub output_node_id: String,
    pub inputs: Vec<PackageDerivationInputRow>,
    pub action_kind: String,
    pub action_version: i64,
    pub engine: String,
    pub status: String,
    pub error: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: i64,
    pub grounded_document: Option<PackageGroundedDocumentRow>,
    pub generated_image: Option<PackageGeneratedImageRow>,
    pub generated_video: Option<PackageGeneratedVideoRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageGroundedDocumentRow {
    pub citations: Value,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageGeneratedImageRow {
    pub intent: String,
    pub style: String,
    pub aspect_ratio: String,
    pub provider: String,
    pub model: String,
    pub revised_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageGeneratedVideoRow {
    pub prompt: String,
    pub aspect_ratio: String,
    pub duration_seconds: i64,
    pub provider: String,
    pub model: String,
    pub operation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageDerivationInputRow {
    pub node_id: Option<String>,
    pub source_identity: String,
    pub source_snapshot: PackageSourceSnapshot,
    pub role: String,
    pub source_revision: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSourceSnapshot {
    pub title: String,
    pub kind: String,
    pub original_url: Option<String>,
    pub platform: Option<String>,
}
