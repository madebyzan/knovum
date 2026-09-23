use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ai::models::ExecutionTargetSelection;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AssistantMode {
    Ask,
    Plan,
    Execute,
}

impl AssistantMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Plan => "plan",
            Self::Execute => "execute",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartAssistantTaskRequest {
    pub space_id: String,
    pub conversation_id: Option<String>,
    pub branch_id: Option<String>,
    pub prompt: String,
    pub mode: AssistantMode,
    pub target: ExecutionTargetSelection,
    pub max_turns: u16,
    pub max_tool_calls: u16,
    pub max_total_tokens: u32,
    pub max_duration_seconds: u32,
    #[serde(default)]
    pub attachments: Vec<AssistantAttachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssistantAttachment {
    pub node_id: String,
    pub source_revision: String,
    pub role: String,
    #[ts(type = "unknown")]
    pub locator: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "AssistantTaskAccepted.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssistantTaskAccepted {
    pub task_id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "AssistantTaskUpdate.ts")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssistantTaskUpdate {
    ContentDelta { task_id: String, text: String },
    Finished { state: String },
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "AssistantConversation.ts")]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub space_id: String,
    pub branch_id: Option<String>,
    pub title: String,
    pub mode: AssistantMode,
    pub updated_at: i64,
}
