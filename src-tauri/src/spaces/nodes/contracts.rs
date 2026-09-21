use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::spaces::knowledge::indexing::KnowledgeChunkInput;
use crate::spaces::structure::SizeAuthority;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InsertNode {
    pub id: String,
    pub kind: String,
    pub initial_x: f64,
    pub initial_y: f64,
    pub width: f64,
    pub height: f64,
    pub size_authority: SizeAuthority,
    #[serde(default)]
    pub appearance: Option<Value>,
    #[serde(default)]
    pub file_subpath: Option<String>,
    pub data: Value,
    pub source_chunks: Vec<KnowledgeChunkInput>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InsertNodesRequest {
    pub space_id: String,
    pub mutation_id: String,
    pub expected_edit_revision: i64,
    pub target_parent_id: Option<String>,
    pub nodes: Vec<InsertNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDataChange {
    pub id: String,
    pub data: Value,
    pub source_chunks: Option<Vec<KnowledgeChunkInput>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodesRequest {
    pub space_id: String,
    pub changes: Vec<NodeDataChange>,
}
