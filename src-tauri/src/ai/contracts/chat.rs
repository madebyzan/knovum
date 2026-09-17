use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::ai::{error::AiResult, transport::SecurityContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    std::pin::Pin<Box<dyn futures::Stream<Item = AiResult<StreamFrame>> + Send>>;

pub trait ChatCapability: Send + Sync {
    fn complete<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ChatRequest,
    ) -> BoxFuture<'a, AiResult<ChatResponse>>;

    fn stream<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ChatRequest,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> BoxFuture<'a, AiResult<StreamFrameStream>>;
}
