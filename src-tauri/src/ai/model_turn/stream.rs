use futures::StreamExt;
use genai::{adapter::AdapterKind, chat::ChatStreamEvent, Client, ModelIden, ServiceTarget};
use tokio_util::sync::CancellationToken;

use crate::ai::{contracts::ModelTurnRequest, transport::SecurityContext};
use crate::app_error::{AppError, AppResult};

use super::{to_genai_options, to_genai_request, wire::response::from_stream_end};

pub(super) async fn stream_turn(
    adapter: AdapterKind,
    context: SecurityContext,
    request: ModelTurnRequest,
    cancellation: CancellationToken,
    deltas: tokio::sync::mpsc::Sender<String>,
) -> AppResult<crate::ai::contracts::ModelTurnResponse> {
    let target = service_target(adapter, &context);
    let options = to_genai_options(&request)
        .with_capture_content(true)
        .with_capture_usage(true)
        .with_capture_tool_calls(true);
    let client = Client::default();
    let response = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(AppError::cancelled("Model turn was cancelled")),
        response = client.exec_chat_stream(target, to_genai_request(&request)?, Some(&options)) => response.map_err(provider_error)?,
    };
    let mut source = response.stream;
    let mut output_bytes = 0;
    loop {
        let event = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err(AppError::cancelled("Model turn was cancelled"));
            }
            event = source.next() => event,
        };
        if let Some(Ok(event)) = &event {
            super::output_budget::account(&mut output_bytes, event)?;
        }
        match event {
            Some(Ok(ChatStreamEvent::Chunk(chunk))) => {
                tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(AppError::cancelled("Model turn was cancelled")),
                    sent = deltas.send(chunk.content) => {
                        if sent.is_err() {
                            return Err(AppError::cancelled("Model turn consumer was released"));
                        }
                    }
                }
            }
            Some(Ok(ChatStreamEvent::End(end))) => return from_stream_end(&context, end),
            Some(Ok(
                ChatStreamEvent::Start
                | ChatStreamEvent::ReasoningChunk(_)
                | ChatStreamEvent::ThoughtSignatureChunk(_)
                | ChatStreamEvent::ToolCallChunk(_),
            )) => {}
            Some(Err(error)) => return Err(provider_error(error)),
            None => {
                return Err(AppError::new(
                    "invalid_response",
                    "Model stream ended without a terminal response",
                ))
            }
        }
    }
}

pub(super) fn service_target(adapter: AdapterKind, context: &SecurityContext) -> ServiceTarget {
    let auth = context
        .api_key
        .as_ref()
        .map_or(genai::resolver::AuthData::None, |key| {
            genai::resolver::AuthData::from_single(key.clone())
        });
    ServiceTarget {
        endpoint: genai::resolver::Endpoint::from_owned(format!(
            "{}/",
            context.endpoint.trim_end_matches('/')
        )),
        auth,
        model: ModelIden::new(adapter, context.model.clone()),
    }
}

pub(super) fn provider_error(error: genai::Error) -> AppError {
    AppError::new("provider_request_failed", error.to_string())
}
