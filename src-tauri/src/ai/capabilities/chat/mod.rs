mod request;
mod telemetry;

use futures::StreamExt;
use std::time::Instant;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use crate::ai::channel::send_channel_event;
use crate::ai::{
    contracts::StreamFrame,
    models::{CompletionOutput, CompletionRequest, CompletionStreamEvent},
};
use crate::foundation::app_error::{AppError, AppResult};

pub(crate) use request::stream_target;

pub async fn complete_cancellable(
    request: CompletionRequest,
    cancellation: CancellationToken,
) -> AppResult<CompletionOutput> {
    let (target, chat_req) = request::prepare(request);
    let response = request::complete_target(&target, &chat_req, &cancellation).await?;
    Ok(CompletionOutput {
        text: response.text,
        provider: response.provider,
        model: response.model,
        usage: response.usage,
    })
}

pub async fn complete_stream(
    request: CompletionRequest,
    events: Channel<CompletionStreamEvent>,
    cancellation: CancellationToken,
) -> AppResult<CompletionOutput> {
    let started_at = Instant::now();
    let prompt_bytes = request.text.len();
    telemetry::stream_request_started(&request.target, prompt_bytes);

    let (target, chat_req) = request::prepare(request);

    let mut stream = stream_target(&target, &chat_req, cancellation).await?;
    telemetry::stream_opened(&target, started_at);

    let mut full_text = String::new();
    let mut chunk_count = 0_u64;
    let mut execution_metadata = None;
    let mut usage = None;

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
            StreamFrame::Usage(value) => {
                let prompt_tokens = value.prompt_tokens;
                let completion_tokens = value.completion_tokens;
                let total_tokens = value.total_tokens;
                send_channel_event(
                    &events,
                    CompletionStreamEvent::Usage {
                        prompt_tokens,
                        completion_tokens,
                        total_tokens,
                    },
                )?;
                usage = Some(crate::ai::contracts::TokenUsage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                });
            }
        }
    }

    if full_text.trim().is_empty() {
        return Err(AppError::new(
            "empty_response",
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
        usage,
    })
}

fn require_execution_metadata(metadata: Option<(String, String)>) -> AppResult<(String, String)> {
    metadata.ok_or_else(|| {
        AppError::new(
            "invalid_response",
            "AI execution stream did not identify its provider and model.",
        )
    })
}
