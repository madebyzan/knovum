use std::sync::Arc;

use futures::{channel::mpsc, future::BoxFuture, SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{AgentKind, ChatRequest, ChatResponse, StreamFrame, StreamFrameStream},
    error::{AiError, AiErrorCode, AiResult},
};

use super::session::AgentSessionUpdate;

pub fn complete<'a>(
    kind: AgentKind,
    request: &'a ChatRequest,
) -> BoxFuture<'a, AiResult<ChatResponse>> {
    Box::pin(async move {
        let mut stream = stream(kind, request, CancellationToken::new()).await?;
        let mut text = String::new();
        while let Some(frame) = stream.next().await {
            if let StreamFrame::ContentDelta(delta) = frame? {
                text.push_str(&delta);
            }
        }
        if text.trim().is_empty() {
            return Err(AiError::new(
                AiErrorCode::EmptyResponse,
                "Agent returned no text",
            ));
        }
        Ok(ChatResponse {
            text: text.trim().to_string(),
            provider: kind.as_str().to_owned(),
            model: super::registry::descriptor(kind).display_name.to_owned(),
            usage: None,
        })
    })
}

pub fn stream<'a>(
    kind: AgentKind,
    request: &'a ChatRequest,
    cancellation: CancellationToken,
) -> BoxFuture<'a, AiResult<StreamFrameStream>> {
    Box::pin(async move {
        let directory =
            std::env::temp_dir().join(format!("knovum-agent-chat-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory)
            .map_err(|error| AiError::internal(error.to_string()))?;
        let prompt = compose_prompt(request);
        let provider = kind.as_str().to_owned();
        let model = super::registry::descriptor(kind).display_name.to_owned();
        let cleanup = directory.clone();
        let (sender, receiver) = mpsc::channel::<AiResult<StreamFrame>>(64);
        let event_sender = Arc::new(std::sync::Mutex::new(sender.clone()));
        let on_update = Arc::new(move |update| {
            let frame = match update {
                AgentSessionUpdate::Thought(text) => Some(StreamFrame::ReasoningDelta(text)),
                AgentSessionUpdate::Message(text) => Some(StreamFrame::ContentDelta(text)),
                AgentSessionUpdate::ToolCall { .. } | AgentSessionUpdate::ToolUpdate { .. } => None,
            };
            if let Some(frame) = frame {
                event_sender
                    .lock()
                    .map_err(|_| AiError::internal("Agent stream sender is unavailable"))?
                    .try_send(Ok(frame))
                    .map_err(|_| AiError::internal("Agent stream consumer is too slow"))?;
            }
            Ok(())
        });

        tokio::spawn(async move {
            let mut sender = sender;
            if sender
                .send(Ok(StreamFrame::Started { provider, model }))
                .await
                .is_ok()
            {
                let result = super::session::run(
                    kind,
                    directory,
                    super::prompt::AgentPrompt::text(prompt),
                    cancellation,
                    on_update,
                    Vec::new(),
                )
                .await;
                if let Err(error) = result {
                    let _ = sender.send(Err(error)).await;
                }
            }
            if let Err(error) = std::fs::remove_dir_all(cleanup) {
                log::warn!("failed to remove Agent chat workspace: {error}");
            }
        });
        Ok(Box::pin(receiver) as StreamFrameStream)
    })
}

fn compose_prompt(request: &ChatRequest) -> String {
    [
        request.system_prompt.as_deref(),
        request.instruction.as_deref(),
        Some(request.text.as_str()),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}
