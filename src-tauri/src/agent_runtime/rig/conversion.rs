use std::collections::BTreeSet;

use rig_agent::{
    agent::run::ModelTurn,
    core::{
        completion::{FinishReason, Message, Usage},
        message::{
            AssistantContent, DocumentSourceKind, Image, ImageMediaType, Message as RigMessage,
            MimeType, Text, ToolCall, ToolFunction, UserContent,
        },
    },
};

use crate::ai::contracts::{
    ModelContentPart, ModelFinishReason, ModelMessage, ModelRole, ModelTurnResponse,
};

pub fn messages(history: Vec<Message>, prompt: Message) -> Vec<ModelMessage> {
    history
        .into_iter()
        .chain(std::iter::once(prompt))
        .map(message)
        .collect()
}

pub fn prompt(message: ModelMessage) -> Result<RigMessage, String> {
    if !matches!(message.role, ModelRole::User) {
        return Err("assistant-prompt-invalid: initial Agent prompt must be a user message".into());
    }
    let content = message
        .content
        .into_iter()
        .map(|part| match part {
            ModelContentPart::Text { text } => Ok(UserContent::Text(Text::new(text))),
            ModelContentPart::Binary {
                media_type, data, ..
            } => {
                let media_type = ImageMediaType::from_mime_type(&media_type).ok_or(
                    "assistant-attachment-unsupported: binary prompt part is not a supported image",
                )?;
                Ok(UserContent::Image(Image {
                    data: DocumentSourceKind::Base64(data),
                    media_type: Some(media_type),
                    detail: None,
                    additional_params: None,
                }))
            }
            _ => Err("assistant-prompt-invalid: initial prompt contains a control part"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RigMessage::User { content })
}

pub fn model_turn(response: ModelTurnResponse, advertised_tools: BTreeSet<String>) -> ModelTurn {
    let choice = response
        .message
        .content
        .into_iter()
        .filter_map(assistant_content)
        .collect();
    let usage = response.usage.map_or_else(Usage::new, |usage| Usage {
        input_tokens: u64::from(usage.prompt_tokens),
        output_tokens: u64::from(usage.completion_tokens),
        total_tokens: u64::from(usage.total_tokens),
        cached_input_tokens: 0,
        cache_creation_input_tokens: 0,
        tool_use_prompt_tokens: 0,
        reasoning_tokens: 0,
    });
    ModelTurn::new(
        None,
        choice,
        usage,
        advertised_tools.clone(),
        advertised_tools,
    )
    .with_identity(response.response_id, None)
    .with_finish_reason(finish_reason(response.finish_reason))
}

fn message(message: Message) -> ModelMessage {
    match message {
        Message::System { content } => model_text(ModelRole::System, content),
        Message::User { content } => ModelMessage {
            role: ModelRole::User,
            content: content.into_iter().filter_map(user_content).collect(),
        },
        Message::Assistant { content, .. } => ModelMessage {
            role: ModelRole::Assistant,
            content: content
                .into_iter()
                .filter_map(rig_assistant_content)
                .collect(),
        },
    }
}

fn model_text(role: ModelRole, text: String) -> ModelMessage {
    ModelMessage {
        role,
        content: vec![ModelContentPart::Text { text }],
    }
}

fn user_content(content: UserContent) -> Option<ModelContentPart> {
    match content {
        UserContent::Text(text) => Some(ModelContentPart::Text { text: text.text }),
        UserContent::ToolResult(tool_reply) => Some(ModelContentPart::ToolResult {
            call_id: tool_reply.call.into_string(),
            name: tool_reply.name,
            result: serde_json::to_value(tool_reply.content).ok()?,
        }),
        UserContent::Image(_) | UserContent::Audio(_) => None,
        UserContent::Video(_) | UserContent::Document(_) => None,
    }
}

fn rig_assistant_content(content: AssistantContent) -> Option<ModelContentPart> {
    match content {
        AssistantContent::Text(text) => Some(ModelContentPart::Text { text: text.text }),
        AssistantContent::ToolCall(call) => Some(ModelContentPart::ToolCall {
            call_id: call.id.into_string(),
            name: call.function.name,
            arguments: call.function.arguments,
            thought_signatures: call.signature.map(|value| vec![value]),
        }),
        AssistantContent::Reasoning(_) | AssistantContent::Image(_) => None,
    }
}

fn assistant_content(content: ModelContentPart) -> Option<AssistantContent> {
    match content {
        ModelContentPart::Text { text } => Some(AssistantContent::Text(Text::new(text))),
        ModelContentPart::ToolCall {
            call_id,
            name,
            arguments,
            thought_signatures,
        } => Some(AssistantContent::ToolCall(
            ToolCall::from_wire(call_id, ToolFunction::new(name, arguments))
                .with_signature(thought_signatures.and_then(|items| items.into_iter().next())),
        )),
        ModelContentPart::Binary { .. } | ModelContentPart::ToolResult { .. } => None,
    }
}

fn finish_reason(reason: ModelFinishReason) -> Option<FinishReason> {
    Some(match reason {
        ModelFinishReason::Completed => FinishReason::Stop,
        ModelFinishReason::ToolCall => FinishReason::ToolCalls,
        ModelFinishReason::MaxTokens => FinishReason::Length,
        ModelFinishReason::ContentFilter => FinishReason::ContentFilter,
        ModelFinishReason::StopSequence => FinishReason::Other("stop_sequence".into()),
        ModelFinishReason::Other => FinishReason::Other("other".into()),
    })
}
