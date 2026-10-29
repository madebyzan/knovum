use std::sync::Mutex;

use tokio_util::sync::CancellationToken;

use super::*;
use crate::ai::{
    contracts::{ExecutionCapability, ExternalAgentKind},
    models::ModelExecutionTarget,
    tasks::{
        completion::{
            tests::{agent_target, input},
            TaskCompletionOutput,
        },
        ExecutionSource,
    },
};

/// Replies with each scripted text in turn and records the instructions it was given.
struct Script {
    replies: Mutex<Vec<&'static str>>,
    instructions: Mutex<Vec<String>>,
}

impl Script {
    fn new(replies: &[&'static str]) -> Self {
        Self {
            replies: Mutex::new(replies.iter().rev().copied().collect()),
            instructions: Mutex::new(Vec::new()),
        }
    }
}

impl TaskExecutor for Script {
    async fn model(
        &self,
        _: ModelExecutionTarget,
        _: TaskInput,
        _: CancellationToken,
    ) -> AppResult<TaskCompletionOutput> {
        unreachable!("the test target is an Agent")
    }

    async fn agent(
        &self,
        kind: ExternalAgentKind,
        input: TaskInput,
        _: CancellationToken,
    ) -> AppResult<TaskCompletionOutput> {
        self.instructions.lock().unwrap().push(input.instruction);
        Ok(TaskCompletionOutput {
            text: self.replies.lock().unwrap().pop().unwrap().into(),
            execution_source: ExecutionSource::Agent { runtime_kind: kind },
            usage: None,
        })
    }
}

fn number(reply: &str) -> Result<u32, String> {
    reply
        .parse()
        .map_err(|_| format!("'{reply}' is not a number"))
}

#[tokio::test]
async fn a_valid_reply_is_parsed_without_a_second_call() {
    let script = Script::new(&["7"]);
    let output = execute_structured(
        &script,
        agent_target(ExecutionCapability::TextGeneration),
        input(),
        number,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(output.value, 7);
    assert_eq!(script.instructions.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn a_rejected_reply_is_asked_again_with_the_reason() {
    let script = Script::new(&["seven", "```json\n7\n```"]);
    let output = execute_structured(
        &script,
        agent_target(ExecutionCapability::TextGeneration),
        input(),
        number,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(output.value, 7);
    let instructions = script.instructions.lock().unwrap();
    assert!(instructions[1].starts_with("Summarize"));
    assert!(instructions[1].contains("'seven' is not a number"));
}

#[tokio::test]
async fn a_second_rejection_fails_the_task() {
    let script = Script::new(&["seven", "eight"]);
    let error = execute_structured(
        &script,
        agent_target(ExecutionCapability::TextGeneration),
        input(),
        number,
        CancellationToken::new(),
    )
    .await
    .err()
    .unwrap();
    assert_eq!(error.code, "invalid_response");
}

#[test]
fn fenced_json_is_unwrapped() {
    assert_eq!(json_body("```json\n{\"a\":1}\n```"), "{\"a\":1}");
    assert_eq!(json_body("  {\"a\":1} "), "{\"a\":1}");
    assert_eq!(json_body("```\n[1]\n```\n"), "[1]");
}
