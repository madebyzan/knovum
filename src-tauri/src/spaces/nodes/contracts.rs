use serde::Deserialize;
use serde_json::Value;

use crate::spaces::knowledge::indexing::KnowledgeChunkInput;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertNode {
    pub id: String,
    pub kind: String,
    pub initial_x: f64,
    pub initial_y: f64,
    pub data: Value,
    pub source_chunks: Vec<KnowledgeChunkInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertNodesRequest {
    pub space_id: String,
    pub base_revision: i64,
    pub document: crate::spaces::canvas_state::SpaceEditorDocument,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeSelectionRequest {
    pub space_id: String,
    pub ids: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEditorMutationResult { pub revision: i64 }
