use genai::chat::{BinarySource, ContentPart, StopReason};

use crate::ai::{
    contracts::{
        ModelBinarySource, ModelContentPart, ModelFinishReason, ModelMessage, ModelRole,
        ModelTurnResponse, TokenUsage,
    },
    transport::SecurityContext,
};
use crate::app_error::{AppError, AppResult};

pub(super) fn from_genai_response(
    context: &SecurityContext,
    response: genai::chat::ChatResponse,
) -> AppResult<ModelTurnResponse> {
    let finish_reason_raw = response
        .stop_reason
        .as_ref()
        .map(|reason| reason.raw().to_owned());
    let finish_reason = finish_reason(response.stop_reason.as_ref());
    let content = response
        .content
        .into_parts()
        .into_iter()
        .map(from_part)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(ModelTurnResponse {
        message: ModelMessage {
            role: ModelRole::Assistant,
            content,
        },
        finish_reason,
        finish_reason_raw,
        usage: usage(response.usage),
        provider: context.provider.clone(),
        model: context.model.clone(),
        response_id: response.response_id,
    })
}

pub(in crate::ai::model_turn) fn from_stream_end(
    context: &SecurityContext,
    end: genai::chat::StreamEnd,
) -> AppResult<ModelTurnResponse> {
    let finish_reason_raw = end
        .captured_stop_reason
        .as_ref()
        .map(|reason| reason.raw().to_owned());
    let finish_reason = finish_reason(end.captured_stop_reason.as_ref());
    let content = end
        .captured_content
        .ok_or_else(|| unsupported("empty streamed response"))?
        .into_parts()
        .into_iter()
        .map(from_part)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(ModelTurnResponse {
        message: ModelMessage {
            role: ModelRole::Assistant,
            content,
        },
        finish_reason,
        finish_reason_raw,
        usage: end.captured_usage.and_then(usage),
        provider: context.provider.clone(),
        model: context.model.clone(),
        response_id: end.captured_response_id,
    })
}

fn from_part(part: ContentPart) -> AppResult<ModelContentPart> {
    match part {
        ContentPart::Text(text) => Ok(ModelContentPart::Text { text }),
        ContentPart::Binary(binary) => match binary.source {
            BinarySource::Base64(data) => Ok(ModelContentPart::Binary {
                media_type: binary.content_type,
                source: ModelBinarySource::Base64 {
                    data: data.to_string(),
                },
                name: binary.name,
            }),
            BinarySource::Url(_) => Err(unsupported("provider returned a remote binary URL")),
        },
        ContentPart::ToolCall(call) => Ok(ModelContentPart::ToolCall {
            call_id: call.call_id,
            name: call.fn_name,
            arguments: call.fn_arguments,
            thought_signatures: call.thought_signatures,
        }),
        ContentPart::ToolResponse(tool_reply) => Ok(ModelContentPart::ToolResult {
            call_id: tool_reply.call_id,
            name: tool_reply.fn_name.unwrap_or_default(),
            result: serde_json::from_str(&tool_reply.content)
                .unwrap_or(serde_json::Value::String(tool_reply.content)),
        }),
        ContentPart::ThoughtSignature(_) => Err(unsupported("standalone thought signature")),
        ContentPart::ReasoningContent(_) => Err(unsupported("reasoning content")),
        ContentPart::Custom(_) => Err(unsupported("provider-specific custom content")),
    }
}

fn unsupported(kind: &str) -> AppError {
    AppError::new(
        "invalid_response",
        format!("ModelTurn cannot preserve {kind}; this provider response is unsupported."),
    )
}

fn finish_reason(reason: Option<&StopReason>) -> ModelFinishReason {
    match reason {
        Some(StopReason::Completed(_)) | None => ModelFinishReason::Completed,
        Some(StopReason::ToolCall(_)) => ModelFinishReason::ToolCall,
        Some(StopReason::MaxTokens(_)) => ModelFinishReason::MaxTokens,
        Some(StopReason::ContentFilter(_)) => ModelFinishReason::ContentFilter,
        Some(StopReason::StopSequence(_)) => ModelFinishReason::StopSequence,
        Some(StopReason::Other(_)) => ModelFinishReason::Other,
    }
}

fn usage(usage: genai::chat::Usage) -> Option<TokenUsage> {
    crate::ai::usage::from_provider_usage(usage)
}
