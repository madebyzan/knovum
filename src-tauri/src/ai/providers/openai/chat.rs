use futures::{channel::mpsc, future::BoxFuture, SinkExt, StreamExt};
use genai::{
    adapter::AdapterKind,
    chat::{ChatOptions, ChatRequest as GenAiChatRequest, ChatStreamEvent},
    resolver::{AuthData, Endpoint},
    Client, ModelIden, ServiceTarget,
};
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{ChatCapability, ChatRequest, ChatResponse, StreamFrame, StreamFrameStream},
    error::{AiError, AiErrorCode, AiResult},
    transport::SecurityContext,
};

pub struct OpenAiChatAdapter {
    pub adapter_kind: AdapterKind,
}

impl OpenAiChatAdapter {
    pub fn new(adapter_kind: AdapterKind) -> Self {
        Self { adapter_kind }
    }

    fn service_target(&self, ctx: &SecurityContext) -> AiResult<ServiceTarget> {
        let endpoint = format!("{}/", ctx.endpoint.trim_end_matches('/'));
        let auth = match &ctx.api_key {
            Some(key) => AuthData::from_single(key.clone()),
            None => AuthData::None,
        };
        Ok(ServiceTarget {
            endpoint: Endpoint::from_owned(endpoint),
            auth,
            model: ModelIden::new(self.adapter_kind, ctx.model.clone()),
        })
    }
}

impl ChatCapability for OpenAiChatAdapter {
    fn complete<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ChatRequest,
    ) -> BoxFuture<'a, AiResult<ChatResponse>> {
        Box::pin(async move {
            let target = self.service_target(ctx)?;
            let chat_req = build_chat_request(req);
            let mut options = ChatOptions::default();
            if let Some(temperature) = req.temperature {
                options = options.with_temperature(temperature);
            }
            if let Some(max) = req.max_tokens {
                options = options.with_max_tokens(max);
            }
            let client = Client::default();
            let response = client
                .exec_chat(target, chat_req, Some(&options))
                .await
                .map_err(|e| AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string()))?;

            let text = response
                .first_text()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    AiError::new(AiErrorCode::EmptyResponse, "AI provider returned no text.")
                })?
                .to_string();

            Ok(ChatResponse {
                text,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
                usage: None,
            })
        })
    }

    fn stream<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ChatRequest,
        cancellation: CancellationToken,
    ) -> BoxFuture<'a, AiResult<StreamFrameStream>> {
        Box::pin(async move {
            let target = self.service_target(ctx)?;
            let chat_req = build_chat_request(req);
            let mut options = ChatOptions::default();
            if let Some(temperature) = req.temperature {
                options = options.with_temperature(temperature);
            }
            if let Some(max) = req.max_tokens {
                options = options.with_max_tokens(max);
            }
            let client = Client::default();
            let response = client
                .exec_chat_stream(target, chat_req, Some(&options))
                .await
                .map_err(|e| AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string()))?;

            let (mut tx, rx) = mpsc::channel(32);
            let provider = ctx.provider.clone();
            let model = ctx.model.clone();
            let mut genai_stream = response.stream;

            tokio::spawn(async move {
                let _ = tx.send(Ok(StreamFrame::Started { provider, model })).await;
                loop {
                    let next_item = tokio::select! {
                        _ = cancellation.cancelled() => {
                            let _ = tx.send(Err(AiError::cancelled("Request was cancelled"))).await;
                            break;
                        }
                        item = genai_stream.next() => item,
                    };
                    match next_item {
                        Some(Ok(ChatStreamEvent::Chunk(c))) => {
                            if tx
                                .send(Ok(StreamFrame::ContentDelta(c.content)))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Some(Ok(ChatStreamEvent::ReasoningChunk(r))) => {
                            if tx
                                .send(Ok(StreamFrame::ReasoningDelta(r.content)))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Some(Ok(ChatStreamEvent::End(_))) | None => break,
                        Some(Err(e)) => {
                            let _ = tx
                                .send(Err(AiError::new(
                                    AiErrorCode::ProviderRequestFailed,
                                    e.to_string(),
                                )))
                                .await;
                            break;
                        }
                        _ => {}
                    }
                }
            });

            Ok(Box::pin(rx) as StreamFrameStream)
        })
    }
}

fn build_chat_request(req: &ChatRequest) -> GenAiChatRequest {
    let text = match &req.instruction {
        Some(inst) if !inst.trim().is_empty() => format!("{}\n\n{}", inst.trim(), req.text.trim()),
        _ => req.text.trim().to_string(),
    };
    let mut chat = GenAiChatRequest::from_user(text);
    if let Some(sys) = &req.system_prompt {
        if !sys.trim().is_empty() {
            chat = chat.with_system(sys.trim());
        }
    }
    chat
}
