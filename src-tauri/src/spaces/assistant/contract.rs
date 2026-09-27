use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ai::contracts::AgentKind;
use crate::ai::models::ExecutionTargetSelection;

#[derive(Clone, Debug, Deserialize, ts_rs::TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum AssistantExecutionTargetSelection {
    Model {
        connection_id: String,
        model_id: String,
        temperature: f64,
    },
    Agent {
        runtime_kind: AgentKind,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
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

#[derive(Clone, Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(optional_fields = nullable)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SubmitAssistantTaskRequest {
    pub space_id: String,
    pub conversation_id: Option<String>,
    pub branch_id: Option<String>,
    pub prompt: String,
    pub target: AssistantExecutionTargetSelection,
    #[serde(default)]
    pub attachments: Vec<AssistantAttachment>,
}

#[derive(Clone, Debug)]
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
    pub attachments: Vec<AssistantAttachment>,
}

impl SubmitAssistantTaskRequest {
    pub fn into_model(self, target: ExecutionTargetSelection) -> StartAssistantTaskRequest {
        StartAssistantTaskRequest {
            space_id: self.space_id,
            conversation_id: self.conversation_id,
            branch_id: self.branch_id,
            prompt: self.prompt,
            mode: AssistantMode::Execute,
            target,
            max_turns: 12,
            max_tool_calls: 32,
            max_total_tokens: 100_000,
            max_duration_seconds: 900,
            attachments: self.attachments,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum AssistantAttachment {
    Node {
        node_id: String,
        source_revision: String,
        role: String,
        #[ts(type = "unknown", optional)]
        locator: Option<serde_json::Value>,
    },
    File {
        path: String,
        source_revision: String,
    },
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/AssistantTaskAccepted.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssistantTaskAccepted {
    pub task_id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/AssistantTaskUpdate.ts")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssistantTaskUpdate {
    ContentDelta {
        task_id: String,
        text: String,
    },
    PermissionRequested {
        session_id: String,
        request_id: String,
        title: String,
        options: Vec<AssistantPermissionOption>,
    },
    Finished {
        state: String,
    },
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/AssistantPermissionOption.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssistantPermissionOption {
    pub id: String,
    pub label: String,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/ConversationSummary.ts")]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub space_id: String,
    pub branch_id: Option<String>,
    pub title: String,
    pub mode: AssistantMode,
    pub updated_at: i64,
}
