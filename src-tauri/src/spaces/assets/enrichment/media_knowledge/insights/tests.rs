use super::*;
use crate::ai::{
    contracts::ExecutionCapability,
    tasks::completion::tests::{agent_target, ExecutorSpy},
};
#[tokio::test]
async fn agent_only_chapters_preserve_transcript_and_domain_timestamps() {
    let knowledge = json!({"schemaVersion":4,"sourceRevision":"source", "transcript":{"language":"en","text":"First Second","subtitleCues":[],"durationMs":4000,"segments":[{"startMs":0,"endMs":2000,"text":"First"},{"startMs":2000,"endMs":4000,"text":"Second"}]}});
    let spy = ExecutorSpy::returns(
        r#"{"summary":"Whole record","chapters":[{"startMs":0,"title":"First","summary":"Topic"},{"startMs":2000,"title":"Second","summary":"Topic"}]}"#,
    );
    let next = generate_with(
        &spy,
        agent_target(ExecutionCapability::TextGeneration),
        &knowledge,
        &Channel::new(|_| Ok(())),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    spy.assert_agent_only();
    assert_eq!(next["transcript"], knowledge["transcript"]);
    assert_eq!(
        next["insights"]["executionSource"],
        json!({"type":"agent","runtimeKind":"codex"})
    );
    assert_eq!(next["insights"]["chapters"][0]["endMs"], 2000);
    assert_eq!(next["insights"]["chapters"][1]["endMs"], 4000);
    assert!(spy.inputs.lock().unwrap()[0].text.contains("[2000ms]"));
    let data = json!({"kind":"audio","title":"Record","fileName":"record.wav","assetPath":"/assets/record.wav","sourceRevision":"source","mediaKnowledge":knowledge});
    let pool = super::super::super::commit::tests::fixture(&data).await;
    let published = super::super::super::commit::tests::publish(
        &pool,
        &data,
        serde_json::Map::from_iter([("mediaKnowledge".into(), next.clone())]),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(published["mediaKnowledge"], next);
    let indexed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_knowledge_chunks WHERE node_id='node'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(indexed > 0);
}
#[tokio::test]
async fn invalid_agent_chapters_leave_existing_transcript_untouched() {
    let knowledge =
        json!({"transcript":{"durationMs":4000,"segments":[{"startMs":0,"text":"First"}]}});
    for response in [
        "not JSON",
        "```json\n{}\n```",
        r#"{"summary":"S","chapters":[{"startMs":1,"title":"T","summary":"S"}]}"#,
        r#"{"summary":"S","chapters":[{"startMs":0,"title":"T"}]}"#,
    ] {
        let spy = ExecutorSpy::returns(response);
        assert!(generate_with(
            &spy,
            agent_target(ExecutionCapability::TextGeneration),
            &knowledge,
            &Channel::new(|_| Ok(())),
            &CancellationToken::new()
        )
        .await
        .is_err());
        spy.assert_agent_only();
        assert!(knowledge.get("insights").is_none());
    }
}
#[tokio::test]
async fn chunked_agent_transcript_uses_the_same_frozen_executor_for_merge() {
    let text = "a".repeat(9000);
    let knowledge = json!({"transcript":{"durationMs":4000,"segments":[{"startMs":0,"text":text},{"startMs":2000,"text":text}]}});
    let spy = ExecutorSpy::returns(
        r#"{"summary":"S","chapters":[{"startMs":0,"title":"T","summary":"S"}]}"#,
    );
    let next = generate_with(
        &spy,
        agent_target(ExecutionCapability::TextGeneration),
        &knowledge,
        &Channel::new(|_| Ok(())),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(spy.inputs.lock().unwrap().len(), 3);
    spy.assert_agent_only();
    assert_eq!(next["insights"]["chapters"][0]["endMs"], 4000);
}

#[tokio::test]
async fn failed_chunk_cancels_and_drains_started_sessions_without_hiding_the_error() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct FailingExecutor {
        started: AtomicUsize,
        settled: AtomicUsize,
    }
    impl TaskExecutor for FailingExecutor {
        async fn model(
            &self,
            _: ai::models::ModelExecutionTarget,
            _: TaskInput,
            _: CancellationToken,
        ) -> ai::error::AiResult<ai::tasks::completion::TaskCompletionOutput> {
            panic!("Agent task must not reach model transport");
        }
        async fn agent(
            &self,
            _: ai::contracts::AgentKind,
            _: TaskInput,
            cancellation: CancellationToken,
        ) -> ai::error::AiResult<ai::tasks::completion::TaskCompletionOutput> {
            let position = self.started.fetch_add(1, Ordering::SeqCst);
            if position == 1 {
                self.settled.fetch_add(1, Ordering::SeqCst);
                return Err(ai::error::AiError::new(
                    ai::error::AiErrorCode::AuthenticationFailed,
                    "Login required",
                ));
            }
            cancellation.cancelled().await;
            self.settled.fetch_add(1, Ordering::SeqCst);
            Err(ai::error::AiError::cancelled("Sibling failed"))
        }
    }
    let executor = FailingExecutor {
        started: AtomicUsize::new(0),
        settled: AtomicUsize::new(0),
    };
    let knowledge = json!({"transcript":{"durationMs":4000,"segments":[{"startMs":0,"text":"a".repeat(9000)},{"startMs":1000,"text":"b".repeat(9000)},{"startMs":2000,"text":"c".repeat(9000)}]}});
    let error = generate_with(
        &executor,
        agent_target(ExecutionCapability::TextGeneration),
        &knowledge,
        &Channel::new(|_| Ok(())),
        &CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert!(error.starts_with("ai-authentication-failed:"), "{error}");
    assert_eq!(executor.started.load(Ordering::SeqCst), 2);
    assert_eq!(executor.settled.load(Ordering::SeqCst), 2);
    assert!(knowledge.get("insights").is_none());
}
