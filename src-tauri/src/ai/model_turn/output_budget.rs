use genai::chat::ChatStreamEvent;

use crate::foundation::app_error::{AppError, AppResult};

// A turn is persisted and may also retain failed partial output. Bound both copies.
pub(super) const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

pub(super) fn account(total: &mut usize, event: &ChatStreamEvent) -> AppResult<()> {
    let bytes = match event {
        ChatStreamEvent::Chunk(chunk)
        | ChatStreamEvent::ReasoningChunk(chunk)
        | ChatStreamEvent::ThoughtSignatureChunk(chunk) => chunk.content.len(),
        ChatStreamEvent::ToolCallChunk(chunk) => serde_json::to_vec(&chunk.tool_call)
            .map_err(|error| AppError::internal(error.to_string()))?
            .len(),
        ChatStreamEvent::Start | ChatStreamEvent::End(_) => 0,
    };
    *total = total.saturating_add(bytes);
    if *total > MAX_OUTPUT_BYTES {
        return Err(AppError::new(
            "invalid_response",
            "Model turn output exceeded the 8 MiB resource limit",
        ));
    }
    Ok(())
}
