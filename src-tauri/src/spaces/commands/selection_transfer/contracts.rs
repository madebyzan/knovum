use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct TransferSelectionRequest {
    pub source_space_id: String,
    pub mutation_id: String,
    pub target_space_id: String,
    pub root_structure_node_ids: Vec<String>,
    pub target_parent_id: Option<String>,
    pub target_order: u32,
    pub expected_source_edit_revision: i64,
    pub expected_target_edit_revision: i64,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct TransferSelectionResponse {
    pub moved_node_ids: Vec<String>,
    pub moved_structure_node_ids: Vec<String>,
    pub source_edit_revision: i64,
    pub target_edit_revision: i64,
}
