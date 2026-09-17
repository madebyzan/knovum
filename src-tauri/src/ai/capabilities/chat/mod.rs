mod telemetry;

use futures::StreamExt;
use std::time::Instant;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{ChatCapability, ChatRequest, StreamFrame},
    error::{send_channel_event, AiError, AiErrorCode, AiResult},
    models::{CompletionOutput, CompletionRequest, CompletionStreamEvent, ExecutionTarget},
    providers::resolve_provider,
    transport::SecurityContext,
};

pub async fn complete(request: CompletionRequest) -> AiResult<CompletionOutput> {
    let (target, chat_req) = prepare_request(request);

    let response = complete_target(&target, &chat_req).await?;
    Ok(CompletionOutput {
        text: response.text,
        provider: response.provider,
        model: response.model,
    })
}

pub async fn complete_cancellable(
    request: CompletionRequest,
    cancellation: CancellationToken,
) -> AiResult<CompletionOutput> {
    let (target, chat_req) = prepare_request(request);
    let mut stream = stream_target(&target, &chat_req, cancellation).await?;
    let mut text = String::new();
    let mut execution_metadata = None;
    while let Some(frame) = stream.next().await {
        match frame? {
            StreamFrame::Started { provider, model } => {
                execution_metadata = Some((provider, model));
            }
            StreamFrame::ContentDelta(delta) => text.push_str(&delta),
            StreamFrame::ReasoningDelta(_) | StreamFrame::Usage(_) => {}
        }
    }
    if text.trim().is_empty() {
        return Err(AiError::new(
            AiErrorCode::EmptyResponse,
            "AI provider returned no text.",
        ));
    }
    let (provider, model) = require_execution_metadata(execution_metadata)?;
    Ok(CompletionOutput {
        text: text.trim().to_owned(),
        provider,
        model,
    })
}

pub async fn complete_stream(
    request: CompletionRequest,
    events: Channel<CompletionStreamEvent>,
    cancellation: CancellationToken,
) -> AiResult<CompletionOutput> {
    let started_at = Instant::now();
    let prompt_bytes = request.text.len();
    telemetry::stream_request_started(&request.target, prompt_bytes);

    let (target, chat_req) = prepare_request(request);

    let mut stream = stream_target(&target, &chat_req, cancellation).await?;
    telemetry::stream_opened(&target, started_at);

    let mut full_text = String::new();
    let mut chunk_count = 0_u64;
    let mut execution_metadata = None;

    while let Some(frame_outcome) = stream.next().await {
        match frame_outcome? {
            StreamFrame::Started { provider, model } => {
                execution_metadata = Some((provider.clone(), model.clone()));
                send_channel_event(&events, CompletionStreamEvent::Started { provider, model })?;
            }
            StreamFrame::ReasoningDelta(delta) => {
                send_channel_event(
                    &events,
                    CompletionStreamEvent::ThinkingDelta { text: delta },
                )?;
            }
            StreamFrame::ContentDelta(delta) => {
                chunk_count += 1;
                if chunk_count == 1 {
                    telemetry::stream_first_text(&target, started_at);
                }
                full_text.push_str(&delta);
                send_channel_event(&events, CompletionStreamEvent::Delta { text: delta })?;
            }
            StreamFrame::Usage(usage) => {
                send_channel_event(
                    &events,
                    CompletionStreamEvent::Usage {
                        prompt_tokens: usage.prompt_tokens,
                        completion_tokens: usage.completion_tokens,
                        total_tokens: usage.total_tokens,
                    },
                )?;
            }
        }
    }

    if full_text.trim().is_empty() {
        return Err(AiError::new(
            AiErrorCode::EmptyResponse,
            "AI provider returned no text.",
        ));
    }

    telemetry::stream_ended(&target, chunk_count, full_text.len(), started_at);
    telemetry::stream_completed(&target, started_at);

    let (provider, model) = require_execution_metadata(execution_metadata)?;
    Ok(CompletionOutput {
        text: full_text,
        provider,
        model,
    })
}

fn require_execution_metadata(metadata: Option<(String, String)>) -> AiResult<(String, String)> {
    metadata.ok_or_else(|| {
        AiError::new(
            AiErrorCode::InvalidResponse,
            "AI execution stream did not identify its provider and model.",
        )
    })
}

fn prepare_request(request: CompletionRequest) -> (ExecutionTarget, ChatRequest) {
    let chat_request = ChatRequest {
        system_prompt: (!request.system_prompt.trim().is_empty()).then_some(request.system_prompt),
        instruction: (!request.instruction.trim().is_empty()).then_some(request.instruction),
        text: request.text,
        temperature: request.target.temperature(),
        max_tokens: request.max_tokens,
    };
    (request.target, chat_request)
}

pub(crate) async fn complete_target(
    target: &ExecutionTarget,
    request: &ChatRequest,
) -> AiResult<crate::ai::contracts::ChatResponse> {
    if let Some(kind) = target.agent_kind() {
        return crate::ai::providers::agents::chat::complete(kind, request).await;
    }
    let context = SecurityContext::from_target(target)?;
    resolve_model_chat(target)?
        .complete(&context, request)
        .await
}

pub(crate) async fn stream_target(
    target: &ExecutionTarget,
    request: &ChatRequest,
    cancellation: CancellationToken,
) -> AiResult<crate::ai::contracts::StreamFrameStream> {
    if let Some(kind) = target.agent_kind() {
        return crate::ai::providers::agents::chat::stream(kind, request, cancellation).await;
    }
    let context = SecurityContext::from_target(target)?;
    resolve_model_chat(target)?
        .stream(&context, request, cancellation)
        .await
}

fn resolve_model_chat(target: &ExecutionTarget) -> AiResult<&'static dyn ChatCapability> {
    let provider = resolve_provider(target.model_provider_kind()?);
    provider
        .as_chat()
        .ok_or_else(|| AiError::capability_not_supported(target.provider(), "chat"))
}
