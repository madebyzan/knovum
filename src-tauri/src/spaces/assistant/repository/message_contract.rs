use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/AssistantMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub role: AssistantMessageRole,
    pub content: AssistantMessageContent,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct AssistantMessageContent {
    pub parts: Vec<crate::ai::contracts::ModelContentPart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AssistantMessageAttachment>,
    #[serde(default)]
    #[ts(type = "Array<unknown>")]
    pub citations: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<AssistantMessageStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/AssistantMessageAttachment.ts")]
#[serde(
    tag = "source",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AssistantMessageAttachment {
    Node {
        title: String,
        node_id: String,
        kind: String,
        thumbnail: Option<String>,
    },
    File {
        title: String,
        kind: String,
        thumbnail: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum AssistantMessageStatus {
    Stopped,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum AssistantMessageRole {
    User,
    Assistant,
    Tool,
}

impl AssistantMessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}
impl std::str::FromStr for AssistantMessageRole {
    type Err = String;
    fn from_str(role: &str) -> Result<Self, Self::Err> {
        match role {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "tool" => Ok(Self::Tool),
            _ => Err("assistant-message-role-invalid".into()),
        }
    }
}
