use std::collections::BTreeSet;

use rig_agent::{
    agent::run::ModelTurn,
    core::{
        completion::{FinishReason, Message, Usage},
        message::{
            AssistantContent, Message as RigMessage, Text, ToolCall, ToolFunction,
            ToolResultContent, UserContent,
        },
    },
};

use crate::ai::contracts::{
    ModelContentPart, ModelFinishReason, ModelMessage, ModelRole, ModelTurnResponse,
};

pub fn messages(history: Vec<Message>, prompt: Message) -> Result<Vec<ModelMessage>, String> {
    let mut result = Vec::new();
    for item in history.into_iter().chain(std::iter::once(prompt)) {
        result.extend(message(item)?);
    }
    Ok(result)
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
            part @ ModelContentPart::Binary { .. } => {
                super::media::to_image(part).map(UserContent::Image)
            }
            _ => Err("assistant-prompt-invalid: initial prompt contains a control part".into()),
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

fn message(message: Message) -> Result<Vec<ModelMessage>, String> {
    match message {
        Message::System { content } => Ok(vec![model_text(ModelRole::System, content)]),
        Message::User { content } => {
            let mut tools = Vec::new();
            let mut materials = Vec::new();
            for part in content {
                match part {
                    UserContent::ToolResult(reply) => {
                        let mut result = Vec::new();
                        for item in reply.content {
                            match item {
                                ToolResultContent::Image(image) => {
                                    materials.push(super::media::from_image(image)?)
                                }
                                item => result.push(item),
                            }
                        }
                        tools.push(ModelContentPart::ToolResult {
                            call_id: reply.call.into_string(),
                            name: reply.name,
                            result: serde_json::to_value(result)
                                .map_err(|error| error.to_string())?,
                        });
                    }
                    UserContent::Text(text) => {
                        materials.push(ModelContentPart::Text { text: text.text })
                    }
                    UserContent::Image(image) => materials.push(super::media::from_image(image)?),
                    _ => {
                        return Err(
                            "assistant-media-unsupported: runtime contains unsupported input"
                                .into(),
                        )
                    }
                }
            }
            let mut messages = Vec::new();
            if !tools.is_empty() {
                messages.push(ModelMessage {
                    role: ModelRole::Tool,
                    content: tools,
                });
            }
            if !materials.is_empty() {
                messages.push(ModelMessage {
                    role: ModelRole::User,
                    content: materials,
                });
            }
            Ok(messages)
        }
        Message::Assistant { content, .. } => Ok(vec![ModelMessage {
            role: ModelRole::Assistant,
            content: content
                .into_iter()
                .filter_map(rig_assistant_content)
                .collect(),
        }]),
    }
}

fn model_text(role: ModelRole, text: String) -> ModelMessage {
    ModelMessage {
        role,
        content: vec![ModelContentPart::Text { text }],
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
