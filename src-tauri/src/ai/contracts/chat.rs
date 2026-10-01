use serde::{Deserialize, Serialize};

use crate::app_error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum StreamFrame {
    Started { provider: String, model: String },
    ReasoningDelta(String),
    ContentDelta(String),
    Usage(TokenUsage),
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub purpose: crate::ai::usage::UsagePurpose,
    pub system_prompt: Option<String>,
    pub instruction: Option<String>,
    pub text: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

pub type StreamFrameStream =
    std::pin::Pin<Box<dyn futures::Stream<Item = AppResult<StreamFrame>> + Send>>;
