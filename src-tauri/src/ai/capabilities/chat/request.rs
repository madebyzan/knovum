use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        ChatRequest, ChatResponse, ModelContentPart, ModelMessage, ModelRole, ModelToolChoice,
        ModelTurnRequest, StreamFrameStream,
    },
    models::{CompletionRequest, ModelExecutionTarget},
};
use crate::foundation::app_error::{AppError, AppResult};

pub(super) fn prepare(request: CompletionRequest) -> (ModelExecutionTarget, ChatRequest) {
    let chat_request = ChatRequest {
        purpose: request.purpose,
        system_prompt: (!request.system_prompt.trim().is_empty()).then_some(request.system_prompt),
        instruction: (!request.instruction.trim().is_empty()).then_some(request.instruction),
        text: request.text,
        temperature: request.target.temperature(),
        max_tokens: request.max_tokens,
    };
    (request.target, chat_request)
}

pub(crate) async fn complete_target(
    target: &ModelExecutionTarget,
    request: &ChatRequest,
    cancellation: &CancellationToken,
) -> AppResult<ChatResponse> {
    let turn = crate::ai::model_turn::execute(target, &model_turn(request), cancellation).await?;
    let text = turn
        .message
        .content
        .into_iter()
        .filter_map(|part| match part {
            ModelContentPart::Text { text } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.trim().is_empty() {
        return Err(AppError::new(
            "empty_response",
            "AI provider returned no text.",
        ));
    }
    Ok(ChatResponse {
        text,
        provider: turn.provider,
        model: turn.model,
        usage: turn.usage,
    })
}

pub(crate) async fn stream_target(
    target: &ModelExecutionTarget,
    request: &ChatRequest,
    cancellation: CancellationToken,
) -> AppResult<StreamFrameStream> {
    crate::ai::model_turn::stream_text(target, &model_turn(request), cancellation).await
}

fn model_turn(request: &ChatRequest) -> ModelTurnRequest {
    let text = match &request.instruction {
        Some(instruction) if !instruction.trim().is_empty() => {
            format!("{}\n\n{}", instruction.trim(), request.text.trim())
        }
        _ => request.text.trim().to_owned(),
    };
    ModelTurnRequest {
        purpose: request.purpose,
        system: request.system_prompt.clone(),
        messages: vec![ModelMessage {
            role: ModelRole::User,
            content: vec![ModelContentPart::Text { text }],
        }],
        tools: Vec::new(),
        tool_choice: ModelToolChoice::None,
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        previous_response_id: None,
    }
}
