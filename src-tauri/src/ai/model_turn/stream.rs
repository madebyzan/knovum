use futures::StreamExt;
use genai::{adapter::AdapterKind, chat::ChatStreamEvent, Client, ModelIden, ServiceTarget};
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{ModelTurnRequest, StreamFrame, StreamFrameStream, TokenUsage},
    error::{AiError, AiErrorCode, AiResult},
    transport::SecurityContext,
};

use super::{to_genai_options, to_genai_request, wire::response::from_stream_end};

pub(super) async fn stream_turn(
    adapter: AdapterKind,
    context: SecurityContext,
    request: ModelTurnRequest,
    cancellation: CancellationToken,
    deltas: tokio::sync::mpsc::UnboundedSender<String>,
) -> AiResult<crate::ai::contracts::ModelTurnResponse> {
    let target = service_target(adapter, &context);
    let options = to_genai_options(&request)
        .with_capture_content(true)
        .with_capture_usage(true)
        .with_capture_tool_calls(true);
    let response = Client::default()
        .exec_chat_stream(target, to_genai_request(&request)?, Some(&options))
        .await
        .map_err(provider_error)?;
    let mut source = response.stream;
    loop {
        let event = tokio::select! {
            _ = cancellation.cancelled() => {
                return Err(AiError::cancelled("Model turn was cancelled"));
            }
            event = source.next() => event,
        };
        match event {
            Some(Ok(ChatStreamEvent::Chunk(chunk))) => {
                let _ = deltas.send(chunk.content);
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
                return Err(AiError::new(
                    AiErrorCode::InvalidResponse,
                    "Model stream ended without a terminal response",
                ))
            }
        }
    }
}

pub(super) async fn stream_text(
    adapter: AdapterKind,
    context: SecurityContext,
    request: ModelTurnRequest,
    cancellation: CancellationToken,
) -> AiResult<StreamFrameStream> {
    if !request.tools.is_empty() {
        return Err(AiError::new(
            AiErrorCode::InvalidRequest,
            "Text streaming does not accept tool definitions.",
        ));
    }
    let target = service_target(adapter, &context);
    let options = to_genai_options(&request)
        .with_capture_usage(true)
        .with_capture_tool_calls(true);
    let response = Client::default()
        .exec_chat_stream(target, to_genai_request(&request)?, Some(&options))
        .await
        .map_err(provider_error)?;
    let mut source = response.stream;
    let (sender, receiver) = futures::channel::mpsc::unbounded();
    tokio::spawn(async move {
        let _ = sender.unbounded_send(Ok(StreamFrame::Started {
            provider: context.provider.clone(),
            model: context.model.clone(),
        }));
        loop {
            let event = tokio::select! {
                _ = cancellation.cancelled() => break,
                event = source.next() => event,
            };
            let Some(event) = event else { break };
            match event.and_then(map_event) {
                Ok(Some(frame)) => {
                    if let StreamFrame::Usage(usage) = &frame {
                        if let Err(error) = crate::ai::usage::record(
                            request.purpose,
                            &context.provider,
                            &context.model,
                            usage,
                        )
                        .await
                        {
                            let _ = sender.unbounded_send(Err(error));
                            break;
                        }
                    }
                    if sender.unbounded_send(Ok(frame)).is_err() {
                        break;
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    let _ = sender.unbounded_send(Err(provider_error(error)));
                    break;
                }
            }
        }
    });
    Ok(Box::pin(receiver))
}

fn service_target(adapter: AdapterKind, context: &SecurityContext) -> ServiceTarget {
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

fn map_event(event: ChatStreamEvent) -> genai::Result<Option<StreamFrame>> {
    let frame = match event {
        ChatStreamEvent::Start | ChatStreamEvent::ThoughtSignatureChunk(_) => None,
        ChatStreamEvent::Chunk(chunk) => Some(StreamFrame::ContentDelta(chunk.content)),
        ChatStreamEvent::ReasoningChunk(chunk) => Some(StreamFrame::ReasoningDelta(chunk.content)),
        ChatStreamEvent::ToolCallChunk(_) => {
            return Err(genai::Error::Internal(
                "Text-only request returned an unexpected tool call".into(),
            ));
        }
        ChatStreamEvent::End(end) => end
            .captured_usage
            .and_then(token_usage)
            .map(StreamFrame::Usage),
    };
    Ok(frame)
}

fn token_usage(usage: genai::chat::Usage) -> Option<TokenUsage> {
    crate::ai::usage::from_provider_usage(usage)
}

fn provider_error(error: genai::Error) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, error.to_string())
}
