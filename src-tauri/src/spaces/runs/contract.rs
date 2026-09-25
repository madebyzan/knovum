use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    Action,
    Workflow,
    Assistant,
    ExternalAgent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
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
pub enum WaitReason {
    Approval,
    ContentReview,
    Executor,
    ExternalReconciliation,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "RunSummary.ts")]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
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
