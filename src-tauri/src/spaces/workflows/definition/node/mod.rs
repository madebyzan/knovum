use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod contracts;
mod identity;

pub use contracts::{JoinPolicy, WorkflowEdge, WorkflowPort, WorkflowValueType};

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum WorkflowNode {
    Input {
        id: String,
        output_type: WorkflowValueType,
    },
    Output {
        id: String,
        input_type: WorkflowValueType,
    },
    Action {
        id: String,
        action_id: String,
        action_version: u16,
        parameters: serde_json::Value,
    },
    Join {
        id: String,
        policy: JoinPolicy,
    },
    Review {
        id: String,
        purpose: String,
    },
}
