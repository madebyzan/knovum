use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum RunKind {
    Action,
    Workflow,
    Assistant,
    ExternalAgent,
    Automation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum RunState {
    Queued,
    Running,
    Waiting,
    Paused,
    Succeeded,
    CompletedWithErrors,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum StepState {
    Pending,
    Ready,
    Running,
    Waiting,
    Paused,
    Succeeded,
    Failed,
    Skipped,
    Blocked,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum WaitReason {
    ContentReview,
    Executor,
    ExternalReconciliation,
    ModelOutcomeUnknown,
    ModelResponseUncommitted,
    ModelUsageUnknown,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/RunObservation.ts")]
#[serde(rename_all = "camelCase")]
pub struct RunObservation {
    pub id: String,
    pub home_space_id: String,
    pub parent_run_id: Option<String>,
    pub retry_of_run_id: Option<String>,
    pub fork_of_run_id: Option<String>,
    pub kind: RunKind,
    pub state: RunState,
    pub state_version: i64,
    pub wait_reason: Option<WaitReason>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub step_attempts: i64,
    pub expanded_steps: i64,
    pub output_bytes: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}

impl RunState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::CompletedWithErrors | Self::Failed | Self::Cancelled
        )
    }
}
