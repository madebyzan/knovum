use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::spaces::knowledge::indexing::KnowledgeChunkInput;
use crate::spaces::structure::{SizingMode, SpaceEditSnapshotResponse};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InsertNode {
    pub id: String,
    pub kind: String,
    pub initial_x: f64,
    pub initial_y: f64,
    pub width: f64,
    pub height: f64,
    pub sizing_mode: SizingMode,
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
pub struct NodeContentChange {
    pub id: String,
    pub expected_data: Value,
    pub data: Value,
    pub source_chunks: Option<Vec<KnowledgeChunkInput>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodesRequest {
    pub space_id: String,
    pub changes: Vec<NodeContentChange>,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CommittedNodeContent {
    pub id: String,
    #[ts(type = "unknown")]
    pub data: Value,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct UpdateNodesResponse {
    pub nodes: Vec<CommittedNodeContent>,
    pub edit_snapshot: Option<SpaceEditSnapshotResponse>,
}
