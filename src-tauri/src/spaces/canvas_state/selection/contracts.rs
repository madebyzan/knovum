use serde::{Deserialize, Serialize};

use super::super::SpaceEditorDocument;
use crate::spaces::nodes::contracts::NodeDataChange;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeleteCanvasSelectionRequest {
    pub(super) space_id: String,
    pub(super) node_ids: Vec<String>,
    pub(super) relation_ids: Vec<String>,
    pub(super) survivor_changes: Vec<NodeDataChange>,
    pub(super) base_revision: i64,
    pub(super) document: SpaceEditorDocument,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanvasSelectionMutationResult {
    pub(super) revision: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RestoreCanvasSelectionRequest {
    pub(super) space_id: String,
    pub(super) node_ids: Vec<String>,
    pub(super) relation_ids: Vec<String>,
    pub(super) survivor_changes: Vec<NodeDataChange>,
    pub(super) base_revision: i64,
    pub(super) document: SpaceEditorDocument,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetImportedCanvasNodesDeletedRequest {
    pub(super) space_id: String,
    pub(super) node_ids: Vec<String>,
    pub(super) relation_ids: Vec<String>,
    pub(super) deleted: bool,
    pub(super) base_revision: i64,
    pub(super) document: SpaceEditorDocument,
}
