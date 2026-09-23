mod node;

pub use node::{
    ItemErrorPolicy, JoinPolicy, PublicationMode, WorkflowEdge, WorkflowNode, WorkflowPort,
    WorkflowValueType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, TS)]
#[ts(export_to = "WorkflowDefinition.ts")]
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

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowNodeLayout {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FailurePolicy {
    StopRun,
    ContinueIndependent,
    CollectErrors,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, TS)]
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

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, TS)]
#[ts(export_to = "WorkflowDraft.ts")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowDraft {
    pub id: String,
    pub space_id: String,
    pub revision: i64,
    pub definition: WorkflowDefinition,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, TS)]
#[ts(export_to = "WorkflowVersion.ts")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowVersion {
    pub workflow_id: String,
    pub space_id: String,
    pub version: i64,
    pub definition_digest: String,
    pub dependency_digest: String,
    pub compiler_version: u16,
    pub definition: WorkflowDefinition,
    pub published_at: i64,
}
