use serde::Serialize;
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

use super::{
    completion::{self, TaskCompletionOutput, TaskInput},
    ExecutionSource, TaskExecutionTarget,
};
use crate::ai::channel::send_channel_event;
use crate::ai::models::{CompletionRequest, CompletionStreamEvent};
use crate::foundation::app_error::AppResult;

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TaskStreamEvent {
    Started {
        execution_source: ExecutionSource,
    },
    ThinkingDelta {
        text: String,
    },
    Delta {
        text: String,
    },
    Usage {
        prompt_tokens: u32,
        completion_tokens: u32,
        total_tokens: u32,
    },
}

impl From<CompletionStreamEvent> for TaskStreamEvent {
    fn from(event: CompletionStreamEvent) -> Self {
        match event {
            CompletionStreamEvent::Started { provider, model } => Self::Started {
                execution_source: ExecutionSource::Model { provider, model },
            },
            CompletionStreamEvent::ThinkingDelta { text } => Self::ThinkingDelta { text },
            CompletionStreamEvent::Delta { text } => Self::Delta { text },
            CompletionStreamEvent::Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            } => Self::Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            },
        }
    }
}

pub(crate) async fn complete(
    app: &AppHandle,
    target: TaskExecutionTarget,
    input: TaskInput,
    events: Channel<TaskStreamEvent>,
    cancellation: CancellationToken,
) -> AppResult<TaskCompletionOutput> {
    match target {
        TaskExecutionTarget::Model(target) => {
            let channel = Channel::new(move |message| {
                let event: CompletionStreamEvent = message.deserialize()?;
                events.send(TaskStreamEvent::from(event))
            });
            crate::ai::capabilities::chat::complete_stream(
                CompletionRequest {
                    target,
                    purpose: input.purpose,
                    system_prompt: input.system_prompt,
                    instruction: input.instruction,
                    text: input.text,
                    max_tokens: input.max_tokens,
                },
                channel,
                cancellation,
            )
            .await
            .map(Into::into)
        }
        TaskExecutionTarget::Agent { runtime_kind } => {
            send_channel_event(
                &events,
                TaskStreamEvent::Started {
                    execution_source: ExecutionSource::Agent { runtime_kind },
                },
            )?;
            let output = completion::complete(
                app,
                TaskExecutionTarget::Agent { runtime_kind },
                input,
                cancellation,
            )
            .await?;
            send_channel_event(
                &events,
                TaskStreamEvent::Delta {
                    text: output.text.clone(),
                },
            )?;
            Ok(output)
        }
    }
}
