use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ai::tasks::TaskTargetSelection;

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CapturePreview {
    pub steps: Vec<CapturedStep>,
    pub inputs: Vec<BoundaryInput>,
    pub selection_groups: Vec<Vec<String>>,
    pub issues: Vec<CaptureIssue>,
    pub candidate: Option<CaptureCandidate>,
    pub source_digest: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CaptureCandidate {
    pub input_slots: Vec<CandidateInputSlot>,
    pub steps: Vec<CandidateStep>,
    pub output_slots: Vec<CandidateOutputSlot>,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CandidateInputSlot {
    pub id: String,
    pub kind: String,
    pub revision_bases: Vec<String>,
    pub min_items: usize,
    pub max_items: usize,
    pub ordered: bool,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CandidateStep {
    pub id: String,
    pub action_id: String,
    pub action_version: u16,
    pub output_kind: String,
    pub execution_spec: RecordedExecutionSpec,
    pub inputs: Vec<CandidateBinding>,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CandidateBinding {
    pub role: String,
    pub position: i64,
    pub revision_basis: String,
    #[serde(flatten)]
    pub source: CandidateSource,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(
    tag = "sourceType",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum CandidateSource {
    InputSlot { input_slot_id: String },
    InputSequence { input_slot_id: String },
    StepOutput { step_id: String },
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CandidateOutputSlot {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub step_id: String,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CapturedStep {
    pub derivation_id: String,
    pub output_node_id: String,
    pub output_kind: String,
    pub action_id: String,
    pub action_version: u16,
    pub execution_spec: RecordedExecutionSpec,
    pub inputs: Vec<CapturedInput>,
    pub selected: bool,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct RecordedExecutionSpec {
    #[ts(type = "Record<string, unknown>")]
    pub parameters: Value,
    pub target: Option<TaskTargetSelection>,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CapturedInput {
    pub node_id: String,
    pub kind: String,
    pub role: String,
    pub position: i64,
    pub source_revision: String,
    pub revision_basis: String,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct BoundaryInput {
    pub node_id: String,
    pub kind: String,
    pub selected: bool,
    pub uses: Vec<BoundaryUse>,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct BoundaryUse {
    pub derivation_id: String,
    pub position: i64,
    pub role: String,
    pub source_revision: String,
    pub revision_basis: String,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CaptureIssue {
    pub node_id: String,
    pub reason: CaptureIssueReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum CaptureIssueReason {
    ResourceUnavailable,
    DerivationIncomplete,
    ExecutionSpecUnavailable,
    DerivationInputsUnavailable,
    SourceUnavailable,
    SourceMismatch,
    SourceIdentityUnavailable,
    ParameterInstanceReference,
    CandidateBindingUnavailable,
    NoDerivedOutputSelected,
}
