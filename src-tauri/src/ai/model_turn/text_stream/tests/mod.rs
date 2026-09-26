use futures::{stream, StreamExt};
use genai::chat::{ChatStreamEvent, StreamChunk, StreamEnd};
use tokio_util::sync::CancellationToken;

use super::text_frames;
use crate::ai::{
    contracts::StreamFrame, error::AiErrorCode, transport::SecurityContext, usage::UsagePurpose,
};

fn context() -> SecurityContext {
    SecurityContext {
        provider: "test".into(),
        model: "test".into(),
        endpoint: "https://unused.invalid".into(),
        api_key: None,
        temperature: 0.0,
    }
}
fn chunk() -> genai::Result<ChatStreamEvent> {
    Ok(ChatStreamEvent::Chunk(StreamChunk {
        content: "partial".into(),
    }))
}

#[tokio::test]
async fn partial_text_then_eof_is_not_success() {
    let mut frames = text_frames(
        stream::iter([chunk()]),
        context(),
        UsagePurpose::Editor,
        CancellationToken::new(),
    );
    assert!(matches!(
        frames.next().await,
        Some(Ok(StreamFrame::Started { .. }))
    ));
    assert!(matches!(
        frames.next().await,
        Some(Ok(StreamFrame::ContentDelta(_)))
    ));
    let error = frames
        .next()
        .await
        .expect("terminal failure")
        .expect_err("unexpected EOF");
    assert_eq!(error.code, AiErrorCode::InvalidResponse);
    assert!(frames.next().await.is_none());
}

#[tokio::test]
async fn partial_text_then_cancel_is_not_success() {
    let cancellation = CancellationToken::new();
    let source = stream::iter([chunk()]).chain(stream::pending());
    let mut frames = text_frames(
        source,
        context(),
        UsagePurpose::Editor,
        cancellation.clone(),
    );
    frames
        .next()
        .await
        .expect("started")
        .expect("started frame");
    frames
        .next()
        .await
        .expect("partial")
        .expect("partial frame");
    cancellation.cancel();
    let error = frames
        .next()
        .await
        .expect("terminal failure")
        .expect_err("cancelled");
    assert_eq!(error.code, AiErrorCode::Cancelled);
    assert!(frames.next().await.is_none());
}

#[tokio::test]
async fn end_without_usage_completes_and_does_not_poll_after_end() {
    let source = stream::iter([chunk(), Ok(ChatStreamEvent::End(StreamEnd::default()))])
        .chain(stream::pending());
    let mut frames = text_frames(
        source,
        context(),
        UsagePurpose::Editor,
        CancellationToken::new(),
    );
    frames
        .next()
        .await
        .expect("started")
        .expect("started frame");
    frames
        .next()
        .await
        .expect("partial")
        .expect("partial frame");
    assert!(frames.next().await.is_none());
}

#[tokio::test]
async fn oversized_output_fails_without_publishing_the_oversized_chunk() {
    let source = stream::iter([Ok(ChatStreamEvent::Chunk(StreamChunk {
        content: "x".repeat(crate::ai::model_turn::output_budget::MAX_OUTPUT_BYTES + 1),
    }))]);
    let mut frames = text_frames(
        source,
        context(),
        UsagePurpose::Editor,
        CancellationToken::new(),
    );
    frames
        .next()
        .await
        .expect("started")
        .expect("started frame");
    let error = frames
        .next()
        .await
        .expect("limit error")
        .expect_err("oversized output");
    assert_eq!(error.code, AiErrorCode::InvalidResponse);
    assert!(error.message.contains("8 MiB"));
    assert!(frames.next().await.is_none());
}
