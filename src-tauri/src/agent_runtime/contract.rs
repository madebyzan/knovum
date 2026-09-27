use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ai::contracts::ModelMessage;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ControlCheckpoint {
    pub schema_version: u16,
    pub runtime: String,
    pub runtime_version: String,
    pub run_id: String,
    pub segment_id: String,
    pub current_turn_id: Option<String>,
    pub control_generation: u64,
    pub catalog_digest: String,
    pub transcript_cursor: i64,
    pub model_turn_count: u16,
    pub tool_call_count: u16,
    pub max_model_turns: u16,
    pub max_tool_calls: u16,
    pub state: Value,
}

#[derive(Clone, Debug, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ControlDirective {
    CallModel {
        turn: usize,
        messages: Vec<ModelMessage>,
    },
    CallTools {
        calls: Vec<RuntimeToolCall>,
    },
    Done {
        output: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeToolReply {
    pub call_id: String,
    pub name: String,
    pub result: Value,
}
