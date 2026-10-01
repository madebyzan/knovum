use futures::{Stream, StreamExt};
use genai::{adapter::AdapterKind, chat::ChatStreamEvent, Client};
use tokio_util::sync::CancellationToken;

use super::{
    stream::{provider_error, service_target},
    to_genai_options, to_genai_request,
};
use crate::ai::{
    contracts::{ModelTurnRequest, StreamFrame, StreamFrameStream, TokenUsage},
    transport::SecurityContext,
    usage::UsagePurpose,
};
use crate::app_error::{AppError, AppResult};

pub(super) async fn stream_text(
    adapter: AdapterKind,
    context: SecurityContext,
    request: ModelTurnRequest,
    cancellation: CancellationToken,
) -> AppResult<StreamFrameStream> {
    if !request.tools.is_empty() {
        return Err(AppError::new(
            "invalid_request",
            "Text streaming does not accept tool definitions.",
        ));
    }
    let target = service_target(adapter, &context);
    let options = to_genai_options(&request)
        .with_capture_usage(true)
        .with_capture_tool_calls(true);
    let client = Client::default();
    let response = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(AppError::cancelled("Text stream was cancelled")),
        response = client.exec_chat_stream(target, to_genai_request(&request)?, Some(&options)) => response.map_err(provider_error)?,
    };
    Ok(text_frames(
        response.stream,
        context,
        request.purpose,
        cancellation,
    ))
}

fn text_frames(
    source: impl Stream<Item = genai::Result<ChatStreamEvent>> + Send + 'static,
    context: SecurityContext,
    purpose: UsagePurpose,
    cancellation: CancellationToken,
) -> StreamFrameStream {
    let started = Ok(StreamFrame::Started {
        provider: context.provider.clone(),
        model: context.model.clone(),
    });
    let frames = futures::stream::unfold(
        Some((Box::pin(source), context, cancellation, 0)),
        move |state| async move {
            let (mut source, context, cancellation, mut output_bytes) = state?;
            loop {
                let event = tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Some((Err(AppError::cancelled("Text stream was cancelled")), None)),
                    event = source.next() => event,
                };
                let Some(event) = event else {
                    return Some((
                        Err(AppError::new(
                            "invalid_response",
                            "Model stream ended without a terminal response",
                        )),
                        None,
                    ));
                };
                if let Ok(event) = &event {
                    if let Err(error) = super::output_budget::account(&mut output_bytes, event) {
                        return Some((Err(error), None));
                    }
                }
                let terminal = matches!(&event, Ok(ChatStreamEvent::End(_)));
                match event.and_then(map_event) {
                    Ok(Some(frame)) => {
                        if let StreamFrame::Usage(usage) = &frame {
                            if let Err(error) = crate::ai::usage::record(
                                purpose,
                                &context.provider,
                                &context.model,
                                usage,
                            )
                            .await
                            {
                                return Some((Err(error), None));
                            }
                        }
                        let next = if terminal {
                            None
                        } else {
                            Some((source, context, cancellation, output_bytes))
                        };
                        return Some((Ok(frame), next));
                    }
                    Ok(None) if terminal => return None,
                    Ok(None) => {}
                    Err(error) => return Some((Err(provider_error(error)), None)),
                }
            }
        },
    );
    Box::pin(futures::stream::once(futures::future::ready(started)).chain(frames))
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

#[cfg(test)]
mod tests;
