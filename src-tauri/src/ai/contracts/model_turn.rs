use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::ai::transport::SecurityContext;
use crate::foundation::app_error::AppResult;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModelBinarySource {
    Base64 { data: String },
    Url { url: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ModelContentPart {
    Text {
        text: String,
    },
    Binary {
        media_type: String,
        source: ModelBinarySource,
        name: Option<String>,
    },
    ToolCall {
        call_id: String,
        name: String,
        #[ts(type = "unknown")]
        arguments: Value,
        thought_signatures: Option<Vec<String>>,
    },
    ToolResult {
        call_id: String,
        name: String,
        #[ts(type = "unknown")]
        result: Value,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ModelMessage {
    pub role: ModelRole,
    pub content: Vec<ModelContentPart>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelToolChoice {
    Auto,
    None,
    Required,
    Tool { name: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTurnRequest {
    pub purpose: crate::ai::usage::UsagePurpose,
    pub system: Option<String>,
    pub messages: Vec<ModelMessage>,
    pub tools: Vec<ModelToolDefinition>,
    pub tool_choice: ModelToolChoice,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub previous_response_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFinishReason {
    Completed,
    ToolCall,
    MaxTokens,
    ContentFilter,
    StopSequence,
    Other,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTurnResponse {
    pub message: ModelMessage,
    pub finish_reason: ModelFinishReason,
    pub finish_reason_raw: Option<String>,
    pub usage: Option<super::TokenUsage>,
    pub provider: String,
    pub model: String,
    pub response_id: Option<String>,
}

pub trait ModelTurnCapability: Send + Sync {
    fn execute_turn<'a>(
        &'a self,
        context: &'a SecurityContext,
        request: &'a ModelTurnRequest,
    ) -> BoxFuture<'a, AppResult<ModelTurnResponse>>;
}
