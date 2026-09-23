use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionInvocationRequest {
    pub invocation_id: String,
    pub idempotency_key: String,
    pub grant_id: String,
    pub space_id: String,
    pub action_id: String,
    pub action_version: u16,
    pub parameters: Value,
    pub input_snapshot: Value,
    pub expected_revisions: Value,
    pub approval_id: Option<String>,
    pub parent_run_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ActionInvocationOutcome {
    QueryResult {
        result: Value,
    },
    CommittedReceipt {
        receipt: Value,
    },
    AcceptedRun {
        run_id: String,
    },
    RequiresApproval {
        approval_id: String,
        plan_digest: String,
    },
}

impl ActionInvocationRequest {
    pub(super) fn validate_identity(&self) -> Result<(), String> {
        if self.invocation_id.trim().is_empty()
            || self.idempotency_key.trim().is_empty()
            || self.grant_id.trim().is_empty()
            || self.space_id.trim().is_empty()
        {
            return Err("SpaceAction invocation identity is required".into());
        }
        Ok(())
    }
}
