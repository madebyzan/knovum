use futures::StreamExt;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

mod protocol;
#[cfg(test)]
mod tests;

use crate::ai::{
    contracts::{ChatRequest, StreamFrame},
    error::{send_channel_event, AiError, AiErrorCode, AiResult},
    models::{CompletionOutput, CompletionStreamEvent, TranslationRequest},
};

pub async fn translate(request: TranslationRequest) -> AiResult<CompletionOutput> {
    if request.text.trim().is_empty() {
        return Ok(CompletionOutput {
            text: String::new(),
            provider: request.execution_target.provider().to_string(),
            model: request.execution_target.model().to_string(),
        });
    }

    let language = protocol::language_name(&request.target_language)?;
    let chat_req = ChatRequest {
        system_prompt: Some(format!(
            "Translate the user text into {language}. Preserve meaning, structure, formatting, and proper nouns. Return only the translation."
        )),
        instruction: None,
        text: request.text,
        temperature: Some(0.0),
        max_tokens: None,
    };

    let response =
        crate::ai::capabilities::chat::complete_target(&request.execution_target, &chat_req)
            .await?;
    Ok(CompletionOutput {
        text: response.text,
        provider: response.provider,
        model: response.model,
    })
}

pub async fn translate_stream(
    request: TranslationRequest,
    events: Channel<CompletionStreamEvent>,
    cancellation: CancellationToken,
) -> AiResult<CompletionOutput> {
    if request.text.trim().is_empty() {
        return Ok(CompletionOutput {
            text: String::new(),
            provider: request.execution_target.provider().to_string(),
            model: request.execution_target.model().to_string(),
        });
    }

    let language = protocol::language_name(&request.target_language)?;
    let chat_req = ChatRequest {
        system_prompt: Some(format!(
            "Translate the user text into {language}. Preserve meaning, structure, formatting, and proper nouns. Return only the translation."
        )),
        instruction: None,
        text: request.text,
        temperature: Some(0.0),
        max_tokens: None,
    };

    let mut stream = crate::ai::capabilities::chat::stream_target(
        &request.execution_target,
        &chat_req,
        cancellation,
    )
    .await?;
    let mut full_text = String::new();

    while let Some(stream_item) = stream.next().await {
        match stream_item? {
            StreamFrame::Started { provider, model } => {
                send_channel_event(&events, CompletionStreamEvent::Started { provider, model })?;
            }
            StreamFrame::ReasoningDelta(delta) => {
                send_channel_event(
                    &events,
                    CompletionStreamEvent::ThinkingDelta { text: delta },
                )?;
            }
            StreamFrame::ContentDelta(delta) => {
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
            "Translation provider returned no text.",
        ));
    }

    Ok(CompletionOutput {
        text: full_text,
        provider: request.execution_target.provider().to_string(),
        model: request.execution_target.model().to_string(),
    })
}
