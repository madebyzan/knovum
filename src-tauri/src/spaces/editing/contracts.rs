use serde::{Deserialize, Serialize};
use crate::spaces::{organization::EditorCommand, canvas_state::contract::SpaceEditorDocument};
use super::{changes::EditChanges, lifecycle_state::LifecycleChanges};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExecuteEditRequest { pub space_id: String, pub mutation_id: String, pub base_revision: i64, pub command: EditorCommand }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReplayEditRequest { pub space_id: String, pub mutation_id: String, pub base_revision: i64, pub receipt_id: String, pub direction: ReplayDirection }
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReplayDirection { Undo, Redo }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReceiptQuery { pub space_id: String, pub mutation_id: String }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct ReceiptChanges { pub version: u8, pub reversible: bool, pub document: EditChanges, pub lifecycle: LifecycleChanges }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditResult {
    pub document: SpaceEditorDocument, pub revision: i64, pub applied_at_revision: i64, pub mutation_id: String,
    pub selection: Vec<String>, pub history_receipt_id: Option<String>,
    pub content: Option<crate::spaces::queries::EditorWorkspaceContent>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditorWorkspace { pub document: SpaceEditorDocument, pub revision: i64, pub content: crate::spaces::queries::EditorWorkspaceContent }
pub(super) struct Receipt {
    pub mutation_id: String, pub request_hash: String, pub revision: i64, pub changes: ReceiptChanges,
    pub selection_before: Vec<String>, pub selection_after: Vec<String>,
}
