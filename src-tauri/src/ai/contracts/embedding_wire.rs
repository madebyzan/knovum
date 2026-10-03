use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingRole {
    Query,
    Document,
}

#[derive(Clone, Debug)]
pub struct EmbeddingExecutionIdentity {
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub deadline_ms: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmbeddingEncodeParams {
    pub request_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub model_spec_id: String,
    pub deadline_ms: u64,
    pub role: EmbeddingRole,
    pub texts: Vec<String>,
    pub embedding_spec_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmbeddingEncodeResult {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub embedding_spec_id: String,
    pub vectors: Vec<Vec<f32>>,
    pub dimensions: u32,
    pub input_tokens: u32,
}
