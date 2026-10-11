use genai::chat::{
    Binary, BinarySource, ChatMessage, ChatOptions, ChatRequest, ChatRole, ContentPart,
    MessageContent, Tool, ToolCall, ToolChoice, ToolResponse,
};

use crate::ai::contracts::{
    ModelBinarySource, ModelContentPart, ModelMessage, ModelRole, ModelToolChoice, ModelTurnRequest,
};
use crate::foundation::app_error::{AppError, AppResult};

pub(crate) fn to_genai_request(request: &ModelTurnRequest) -> AppResult<ChatRequest> {
    let messages = request
        .messages
        .iter()
        .map(to_message)
        .collect::<AppResult<_>>()?;
    let tools = request
        .tools
        .iter()
        .map(|tool| {
            Tool::new(tool.name.clone())
                .with_description(tool.description.clone())
                .with_schema(tool.input_schema.clone())
                .with_strict(true)
        })
        .collect::<Vec<_>>();
    Ok(ChatRequest {
        system: request.system.clone(),
        messages,
        tools: (!tools.is_empty()).then_some(tools),
        previous_response_id: request.previous_response_id.clone(),
        store: request.previous_response_id.as_ref().map(|_| true),
    })
}

fn to_message(message: &ModelMessage) -> AppResult<ChatMessage> {
    let content = message
        .content
        .iter()
        .map(to_part)
        .collect::<AppResult<Vec<_>>>()?;
    let role = match message.role {
        ModelRole::System => ChatRole::System,
        ModelRole::User => ChatRole::User,
        ModelRole::Assistant => ChatRole::Assistant,
        ModelRole::Tool => ChatRole::Tool,
    };
    Ok(ChatMessage::new(role, MessageContent::from_parts(content)))
}

fn to_part(part: &ModelContentPart) -> AppResult<ContentPart> {
    match part {
        ModelContentPart::Text { text } => Ok(ContentPart::Text(text.clone())),
        ModelContentPart::Binary {
            media_type,
            source,
            name,
        } => Ok(ContentPart::Binary(Binary::new(
            media_type.clone(),
            match source {
                ModelBinarySource::Base64 { data } => BinarySource::Base64(data.clone().into()),
                ModelBinarySource::Url { url } => BinarySource::Url(url.clone()),
            },
            name.clone(),
        ))),
        ModelContentPart::ToolCall {
            call_id,
            name,
            arguments,
            thought_signatures,
        } => Ok(ContentPart::ToolCall(ToolCall {
            call_id: call_id.clone(),
            fn_name: name.clone(),
            fn_arguments: arguments.clone(),
            thought_signatures: thought_signatures.clone(),
        })),
        ModelContentPart::ToolResult {
            call_id,
            name,
            result: tool_reply,
        } => Ok(ContentPart::ToolResponse(ToolResponse {
            call_id: call_id.clone(),
            fn_name: Some(name.clone()),
            content: serde_json::to_string(tool_reply)
                .map_err(|error| AppError::new("invalid_request", error.to_string()))?,
        })),
    }
}

pub(crate) fn to_genai_options(request: &ModelTurnRequest) -> ChatOptions {
    let mut options = ChatOptions::default().with_tool_choice(match &request.tool_choice {
        ModelToolChoice::Auto => ToolChoice::Auto,
        ModelToolChoice::None => ToolChoice::None,
        ModelToolChoice::Required => ToolChoice::Required,
        ModelToolChoice::Tool { name } => ToolChoice::tool(name.clone()),
    });
    if let Some(temperature) = request.temperature {
        options = options.with_temperature(temperature);
    }
    if let Some(max_tokens) = request.max_tokens {
        options = options.with_max_tokens(max_tokens);
    }
    options
}
