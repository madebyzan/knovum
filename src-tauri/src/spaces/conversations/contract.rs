use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ai::{models::ModelTargetSelection, tasks::TaskTargetSelection};

#[derive(Clone, Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(optional_fields = nullable)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SubmitConversationTaskRequest {
    pub space_id: String,
    pub conversation_id: Option<String>,
    pub branch_id: Option<String>,
    pub prompt: String,
    pub target: TaskTargetSelection,
    #[serde(default)]
    pub attachments: Vec<ConversationAttachment>,
    pub extensions: crate::extensions::contracts::AgentExtensionSelection,
}

#[derive(Clone, Debug)]
pub struct StartBuiltinAgentTaskRequest {
    pub space_id: String,
    pub conversation_id: Option<String>,
    pub branch_id: Option<String>,
    pub prompt: String,
    pub target: ModelTargetSelection,
    pub max_turns: u16,
    pub max_tool_calls: u16,
    pub max_total_tokens: u32,
    pub max_duration_seconds: u32,
    pub attachments: Vec<ConversationAttachment>,
    pub extensions: crate::extensions::contracts::AgentExtensionSelection,
}

impl SubmitConversationTaskRequest {
    pub fn into_model(self, target: ModelTargetSelection) -> StartBuiltinAgentTaskRequest {
        StartBuiltinAgentTaskRequest {
            space_id: self.space_id,
            conversation_id: self.conversation_id,
            branch_id: self.branch_id,
            prompt: self.prompt,
            target,
            max_turns: 12,
            max_tool_calls: 32,
            max_total_tokens: 100_000,
            max_duration_seconds: 900,
            attachments: self.attachments,
            extensions: self.extensions,
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
pub enum ConversationAttachment {
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
#[ts(export_to = "../../../features/spaces/contracts/generated/ConversationTaskAccepted.ts")]
#[serde(rename_all = "camelCase")]
pub struct ConversationTaskAccepted {
    pub task_id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/ConversationTaskUpdate.ts")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConversationTaskUpdate {
    ContentDelta {
        task_id: String,
        text: String,
    },
    ContentBoundary,
    Finished {
        state: ConversationTaskFinishedState,
    },
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ConversationTaskFinishedState {
    Waiting,
    Succeeded,
    CompletedWithErrors,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/Conversation.ts")]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub space_id: String,
    pub branch_id: Option<String>,
    pub title: String,
    pub updated_at: i64,
}

#[cfg(test)]
#[path = "tests/contract.rs"]
mod tests;
