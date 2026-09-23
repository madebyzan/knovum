use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

mod contracts;
mod identity;

pub use contracts::{
    ItemErrorPolicy, JoinPolicy, PublicationMode, WorkflowEdge, WorkflowPort, WorkflowValueType,
};

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, TS)]
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
        #[ts(type = "Record<string, unknown>")]
        parameters: serde_json::Value,
    },
    Condition {
        id: String,
        expression: String,
    },
    Parallel {
        id: String,
    },
    Join {
        id: String,
        policy: JoinPolicy,
    },
    ForEach {
        id: String,
        max_items: u32,
        parallelism: u16,
        error_policy: ItemErrorPolicy,
    },
    Iterate {
        id: String,
        max_iterations: u16,
        stop_expression: String,
    },
    Subworkflow {
        id: String,
        workflow_id: String,
        version: u32,
    },
    HumanInput {
        id: String,
        #[ts(type = "Record<string, unknown>")]
        schema: serde_json::Value,
    },
    Review {
        id: String,
        purpose: String,
    },
    Delay {
        id: String,
        duration_seconds: u64,
    },
    DataTransform {
        id: String,
        expression: String,
    },
    ExternalTool {
        id: String,
        binding_id: String,
        schema_fingerprint: String,
    },
    DelegateTask {
        id: String,
        runtime_id: String,
    },
    Publish {
        id: String,
        mode: PublicationMode,
        destination: Option<String>,
    },
}
