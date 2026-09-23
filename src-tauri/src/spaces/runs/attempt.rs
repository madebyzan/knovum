use serde_json::Value;

#[derive(Clone, Debug)]
pub struct AttemptClaim {
    pub attempt_id: String,
    pub run_id: String,
    pub step_instance_id: String,
    pub execution_epoch: i64,
    pub attempt_number: i64,
    pub home_space_id: String,
    pub node_kind: String,
    pub configuration: Value,
    pub resolved_input: Value,
    pub previous_output: Option<Value>,
    pub policy_snapshot: Value,
    pub budget: Value,
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
