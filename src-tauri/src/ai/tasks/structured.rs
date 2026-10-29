//! A task whose answer is JSON of one known shape.
//!
//! The instruction carries the JSON Schema derived from the Rust type the caller parses into, so
//! the shape a model is asked for and the shape that is accepted cannot drift. A reply that does
//! not parse is rejected once more with the exact reason, then the task fails: one malformed reply
//! must not become a wrong result, and a model that cannot produce the shape twice will not on a
//! third try.

use schemars::JsonSchema;
use tokio_util::sync::CancellationToken;

use super::completion::{execute, TaskExecutor, TaskInput};
use super::TaskExecutionTarget;
use crate::ai::contracts::TokenUsage;
use crate::foundation::app_error::{AppError, AppResult};

pub(crate) struct StructuredOutput<T> {
    pub value: T,
    /// Usage of every attempt, the rejected one included: both were paid for.
    pub usage: Vec<TokenUsage>,
}

/// The sentence that tells a model to reply with JSON of `T`'s shape.
pub(crate) fn schema_instruction<T: JsonSchema>() -> String {
    let schema = serde_json::to_string(&schemars::schema_for!(T)).unwrap_or_default();
    format!("Reply with a single JSON object and nothing else: no Markdown, no commentary. It must match this JSON Schema exactly, without additional properties: {schema}")
}

/// Runs `input` and parses the reply with `parse`, asking once more with the rejection reason
/// when the first reply does not parse.
pub(crate) async fn execute_structured<T>(
    executor: &impl TaskExecutor,
    target: TaskExecutionTarget,
    input: TaskInput,
    parse: impl Fn(&str) -> Result<T, String>,
    cancellation: CancellationToken,
) -> AppResult<StructuredOutput<T>> {
    let first = execute(
        executor,
        target.clone(),
        input.clone(),
        cancellation.clone(),
    )
    .await?;
    let mut usage = Vec::from_iter(first.usage.clone());
    let rejection = match parse(json_body(&first.text)) {
        Ok(value) => return Ok(StructuredOutput { value, usage }),
        Err(rejection) => rejection,
    };
    let retry = TaskInput {
        instruction: format!(
            "{}\n\nYour previous reply was rejected: {rejection}\nReply again, following the instruction exactly.",
            input.instruction
        ),
        ..input
    };
    let second = execute(executor, target, retry, cancellation).await?;
    usage.extend(second.usage.clone());
    let value = parse(json_body(&second.text)).map_err(|rejection| {
        AppError::new(
            "invalid_response",
            format!("The model did not return the requested structure: {rejection}"),
        )
    })?;
    Ok(StructuredOutput { value, usage })
}

/// The JSON inside a reply: models often fence it in Markdown despite being told not to.
pub(crate) fn json_body(reply: &str) -> &str {
    let reply = reply.trim();
    let Some(fenced) = reply.strip_prefix("```") else {
        return reply;
    };
    let body = fenced.split_once('\n').map_or("", |(_, body)| body);
    body.trim_end().strip_suffix("```").unwrap_or(body).trim()
}

#[cfg(test)]
#[path = "tests/structured.rs"]
mod tests;
