use std::path::PathBuf;

use serde::Serialize;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{ExecutionSource, TaskExecutionTarget};
use crate::ai::{
    capabilities::{chat, vision},
    error::{AiError, AiResult},
    models::{CompletionOutput, CompletionRequest, ImageAnalysisRequest, ModelExecutionTarget},
    usage::UsagePurpose,
};

#[derive(Clone, Debug)]
pub struct TaskInput {
    pub purpose: UsagePurpose,
    pub system_prompt: String,
    pub instruction: String,
    pub text: String,
    pub max_tokens: Option<u32>,
    // Constructed only after the owning feature authorizes and prepares its actual source.
    pub image: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompletionOutput {
    pub text: String,
    pub execution_source: ExecutionSource,
    pub usage: Option<crate::ai::contracts::TokenUsage>,
}

impl From<CompletionOutput> for TaskCompletionOutput {
    fn from(value: CompletionOutput) -> Self {
        Self {
            text: value.text,
            execution_source: ExecutionSource::Model {
                provider: value.provider,
                model: value.model,
            },
            usage: value.usage,
        }
    }
}

pub(crate) trait TaskExecutor: Sync {
    fn model(
        &self,
        target: ModelExecutionTarget,
        input: TaskInput,
        cancellation: CancellationToken,
    ) -> impl std::future::Future<Output = AiResult<TaskCompletionOutput>> + Send;
    fn agent(
        &self,
        kind: crate::ai::contracts::AgentKind,
        input: TaskInput,
        cancellation: CancellationToken,
    ) -> impl std::future::Future<Output = AiResult<TaskCompletionOutput>> + Send;
}

pub(crate) async fn execute(
    executor: &impl TaskExecutor,
    target: TaskExecutionTarget,
    input: TaskInput,
    cancellation: CancellationToken,
) -> AiResult<TaskCompletionOutput> {
    if cancellation.is_cancelled() {
        return Err(AiError::cancelled("AI task was cancelled"));
    }
    let result = match target {
        TaskExecutionTarget::Model(target) => {
            executor.model(target, input, cancellation.clone()).await
        }
        TaskExecutionTarget::Agent { runtime_kind } => {
            executor
                .agent(runtime_kind, input, cancellation.clone())
                .await
        }
    }?;
    if cancellation.is_cancelled() {
        return Err(AiError::cancelled("AI task was cancelled"));
    }
    Ok(result)
}

pub(crate) struct ProductExecutor<'a>(pub &'a AppHandle);

impl TaskExecutor for ProductExecutor<'_> {
    async fn model(
        &self,
        target: ModelExecutionTarget,
        input: TaskInput,
        cancellation: CancellationToken,
    ) -> AiResult<TaskCompletionOutput> {
        let output = if let Some(path) = input.image {
            vision::analyze_image(
                self.0,
                ImageAnalysisRequest {
                    target,
                    source: path,
                    system_prompt: input.system_prompt,
                    instruction: format!("{}\n{}", input.instruction, input.text),
                    max_tokens: input.max_tokens,
                },
                &cancellation,
            )
            .await?
        } else {
            chat::complete_cancellable(
                CompletionRequest {
                    target,
                    purpose: input.purpose,
                    system_prompt: input.system_prompt,
                    instruction: input.instruction,
                    text: input.text,
                    max_tokens: input.max_tokens,
                },
                cancellation,
            )
            .await?
        };
        Ok(output.into())
    }

    async fn agent(
        &self,
        kind: crate::ai::contracts::AgentKind,
        input: TaskInput,
        cancellation: CancellationToken,
    ) -> AiResult<TaskCompletionOutput> {
        let text =
            crate::external_agents::task::complete(self.0, kind, input, cancellation).await?;
        Ok(TaskCompletionOutput {
            text,
            execution_source: ExecutionSource::Agent { runtime_kind: kind },
            usage: None,
        })
    }
}

pub async fn complete(
    app: &AppHandle,
    target: TaskExecutionTarget,
    input: TaskInput,
    cancellation: CancellationToken,
) -> AiResult<TaskCompletionOutput> {
    execute(&ProductExecutor(app), target, input, cancellation).await
}

#[cfg(test)]
pub(crate) mod tests;
