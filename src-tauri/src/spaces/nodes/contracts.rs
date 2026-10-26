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

/// Content creation appends to the Space; it carries no caller revision because the base
/// structure is the one loaded inside its own write transaction.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InsertNodesRequest {
    pub space_id: String,
    pub mutation_id: String,
    pub target_parent_id: Option<String>,
    pub nodes: Vec<InsertNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeContentChange {
    pub id: String,
    pub expected_content: Value,
    pub data: Value,
    pub source_chunks: Option<Vec<KnowledgeChunkInput>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodesRequest {
    pub space_id: String,
    pub changes: Vec<NodeContentChange>,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CommittedNodeContent {
    pub id: String,
    #[ts(type = "unknown")]
    pub data: Value,
}

/// What a creation committed, which is also how a replay of the same mutation is answered: each
/// node as the Space stored it — validated and revisioned — and the edit revision it reached. The
/// caller already holds everything else, so nothing about the rest of the Space is reloaded.
#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CreatedNodes {
    pub edit_revision: i64,
    pub nodes: Vec<CommittedNodeContent>,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct UpdateNodesResponse {
    pub nodes: Vec<CommittedNodeContent>,
    pub edit_snapshot: Option<SpaceEditSnapshotResponse>,
}
