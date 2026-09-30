use super::*;
use crate::ai::{
    contracts::{AgentKind, ExecutionCapability},
    error::AiErrorCode,
    settings,
    tasks::TaskTargetSelection,
};
use serde_json::json;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

pub(crate) struct ExecutorSpy {
    pub(crate) agent_calls: AtomicUsize,
    pub(crate) model_calls: AtomicUsize,
    pub(crate) inputs: Mutex<Vec<TaskInput>>,
    response: AiResult<String>,
    cancel_on_return: bool,
}

impl ExecutorSpy {
    pub(crate) fn returns(text: impl Into<String>) -> Self {
        Self::new(Ok(text.into()), false)
    }
    fn new(response: AiResult<String>, cancel_on_return: bool) -> Self {
        Self {
            agent_calls: AtomicUsize::new(0),
            model_calls: AtomicUsize::new(0),
            inputs: Mutex::new(Vec::new()),
            response,
            cancel_on_return,
        }
    }
    pub(crate) fn assert_agent_only(&self) {
        assert!(self.agent_calls.load(Ordering::SeqCst) > 0);
        assert_eq!(self.model_calls.load(Ordering::SeqCst), 0);
    }
}
impl TaskExecutor for ExecutorSpy {
    async fn model(
        &self,
        target: ModelExecutionTarget,
        _: TaskInput,
        _: CancellationToken,
    ) -> AiResult<TaskCompletionOutput> {
        self.model_calls.fetch_add(1, Ordering::SeqCst);
        Ok(TaskCompletionOutput {
            text: self.response.clone()?,
            execution_source: ExecutionSource::Model {
                provider: target.provider,
                model: target.model,
            },
            usage: None,
        })
    }
    async fn agent(
        &self,
        kind: AgentKind,
        input: TaskInput,
        cancellation: CancellationToken,
    ) -> AiResult<TaskCompletionOutput> {
        self.agent_calls.fetch_add(1, Ordering::SeqCst);
        self.inputs.lock().unwrap().push(input);
        if self.cancel_on_return {
            cancellation.cancel();
        }
        Ok(TaskCompletionOutput {
            text: self.response.clone()?,
            execution_source: ExecutionSource::Agent { runtime_kind: kind },
            usage: None,
        })
    }
}

pub(crate) fn agent_target(capability: ExecutionCapability) -> TaskExecutionTarget {
    let settings = serde_json::from_value(json!({
        "connections":[], "desktopAgents":[{"kind":"codex","enabled":true}],
        "defaults":{"defaultTarget":{"type":"agent","runtimeKind":"codex"}, "writing":null,"knowledge":null,"visualUnderstanding":null,"translation":null,"imageGeneration":null,"videoGeneration":null}
    })).unwrap();
    settings::resolve_task_settings(
        settings,
        TaskTargetSelection::Agent {
            runtime_kind: AgentKind::Codex,
        },
        capability,
    )
    .unwrap()
}

pub(crate) fn input() -> TaskInput {
    TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Knowledge,
        system_prompt: String::new(),
        instruction: "Summarize".into(),
        text: "Evidence".into(),
        max_tokens: None,
        image: None,
    }
}

#[tokio::test]
async fn agent_failure_never_enters_credentials_or_provider_transport() {
    for code in [
        AiErrorCode::AgentUnavailable,
        AiErrorCode::AuthenticationFailed,
        AiErrorCode::ProviderRequestFailed,
        AiErrorCode::InvalidResponse,
        AiErrorCode::Cancelled,
    ] {
        let spy = ExecutorSpy::new(
            Err(AiError::new(
                code,
                "Unavailable / quota exhausted / invalid result",
            )),
            false,
        );
        let error = execute(
            &spy,
            agent_target(ExecutionCapability::TextGeneration),
            input(),
            CancellationToken::new(),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(spy.agent_calls.load(Ordering::SeqCst), 1);
        spy.assert_agent_only();
    }
}

#[tokio::test]
async fn cancellation_invalidates_an_otherwise_valid_late_result() {
    let spy = ExecutorSpy::new(Ok("result".into()), true);
    let error = execute(
        &spy,
        agent_target(ExecutionCapability::TextGeneration),
        input(),
        CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, AiErrorCode::Cancelled);
    spy.assert_agent_only();
}

#[tokio::test]
async fn pre_cancelled_task_does_not_start_an_executor() {
    let token = CancellationToken::new();
    token.cancel();
    let spy = ExecutorSpy::returns("result");
    assert_eq!(
        execute(
            &spy,
            agent_target(ExecutionCapability::TextGeneration),
            input(),
            token
        )
        .await
        .unwrap_err()
        .code,
        AiErrorCode::Cancelled
    );
    assert_eq!(spy.agent_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn direct_model_dispatch_still_uses_the_explicit_user_account() {
    let spy = ExecutorSpy::returns("model result");
    let target = TaskExecutionTarget::Model(ModelExecutionTarget {
        provider: "openai".into(),
        model: "text".into(),
        base_url: "https://api.openai.com/v1".into(),
        secret_key: "integrations.ai.connections.user.api-key".into(),
        temperature: 0.2,
    });
    let result = execute(&spy, target, input(), CancellationToken::new())
        .await
        .unwrap();
    assert!(matches!(
        result.execution_source,
        ExecutionSource::Model { .. }
    ));
    assert_eq!(spy.agent_calls.load(Ordering::SeqCst), 0);
    assert_eq!(spy.model_calls.load(Ordering::SeqCst), 1);
}
