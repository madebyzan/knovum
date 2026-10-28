use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/ConversationMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    pub id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub role: ConversationMessageRole,
    pub content: ConversationMessageContent,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ConversationMessageContent {
    pub parts: Vec<crate::ai::contracts::ModelContentPart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<ConversationMessageAttachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<ConversationMessageOutput>,
    #[serde(default)]
    #[ts(type = "Array<unknown>")]
    pub citations: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<ConversationMessageStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ConversationMessageOutput {
    pub node_id: String,
    pub title: String,
    pub kind: String,
    /// Where the Agent wrote the deliverable, relative to its working directory: the destination
    /// its answer links to. Publication empties that directory, so a link follows this to the node
    /// the file became. Absent for what an Agent made through Space Actions, which has no file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/ConversationMessageAttachment.ts")]
#[serde(
    tag = "source",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ConversationMessageAttachment {
    /// A canvas node: a reference, drawn from the node as it is wherever the conversation shows it.
    Node {
        title: String,
        node_id: String,
        kind: String,
    },
    /// A file handed in from outside the Space. Nothing else keeps it, so the message keeps its
    /// picture.
    File {
        title: String,
        kind: String,
        thumbnail: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ConversationMessageStatus {
    Stopped,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ConversationMessageRole {
    User,
    Assistant,
    Tool,
}

impl ConversationMessageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}
impl std::str::FromStr for ConversationMessageRole {
    type Err = String;
    fn from_str(role: &str) -> Result<Self, Self::Err> {
        match role {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "tool" => Ok(Self::Tool),
            _ => Err("conversation-message-role-invalid".into()),
        }
    }
}
