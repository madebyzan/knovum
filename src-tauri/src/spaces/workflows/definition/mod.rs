mod node;

pub use node::{JoinPolicy, WorkflowEdge, WorkflowNode, WorkflowPort, WorkflowValueType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowDefinition {
    pub schema_version: u16,
    pub title: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    #[serde(default)]
    pub layout: HashMap<String, WorkflowNodeLayout>,
    pub failure_policy: FailurePolicy,
    pub limits: WorkflowLimits,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowNodeLayout {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailurePolicy {
    StopRun,
    ContinueIndependent,
    CollectErrors,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowLimits {
    pub max_expanded_steps: u32,
    pub max_parallelism: u16,
    pub max_duration_seconds: u64,
    pub max_total_tokens: u64,
    pub max_step_attempts: u32,
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
}
