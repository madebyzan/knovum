use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::spaces::authorization::RunScope;

#[derive(Clone, Debug)]
pub struct AttemptClaim {
    pub attempt_id: String,
    pub run_id: String,
    pub step_instance_id: String,
    pub execution_epoch: i64,
    pub attempt_number: i64,
    pub scope: RunScope,
    pub node_kind: String,
    pub configuration: Value,
    pub resolved_input: Value,
    pub policy_snapshot: Value,
    pub budget: Value,
    pub cancellation: CancellationToken,
}

#[derive(Debug)]
pub(crate) enum StepOutcome {
    Completed {
        output: Value,
        selected_branch: Option<String>,
    },
    Waiting {
        reason: super::WaitReason,
        checkpoint: Value,
    },
}
