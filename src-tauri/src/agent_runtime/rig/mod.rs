mod conversion;
mod media;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use rig_agent::{
    agent::run::{AgentRun, AgentRunStep, ModelTurnOutcome},
    core::message::{ToolCallId, ToolResult, ToolResultContent, UserContent},
};

use crate::{
    agent_runtime::{ControlCheckpoint, ControlDirective, RuntimeToolCall, RuntimeToolReply},
    ai::contracts::ModelTurnResponse,
};

const RUNTIME: &str = "rig-agent";
const RUNTIME_VERSION: &str = "0.42.0";

pub fn start(
    prompt: crate::ai::contracts::ModelMessage,
    max_turns: u16,
    max_tool_calls: u16,
    run_id: String,
    segment_id: String,
    catalog_digest: String,
) -> Result<ControlCheckpoint, String> {
    let run = AgentRun::new(conversion::prompt(prompt)?).max_turns(usize::from(max_turns));
    checkpoint(
        run,
        run_id,
        segment_id,
        catalog_digest,
        max_turns,
        max_tool_calls,
    )
}

pub fn next(checkpoint: &mut ControlCheckpoint) -> Result<ControlDirective, String> {
    verify(checkpoint)?;
    let mut run = restore(checkpoint)?;
    let directive = match run.next_step().map_err(|error| error.to_string())? {
        AgentRunStep::CallModel {
            prompt,
            history,
            turn,
        } => {
            checkpoint.current_turn_id = Some(format!("{}:{turn}", checkpoint.segment_id));
            ControlDirective::CallModel {
                turn,
                messages: conversion::messages(history, prompt)?,
            }
        }
        AgentRunStep::CallTools { calls } => ControlDirective::CallTools {
            calls: calls
                .into_iter()
                .map(|call| RuntimeToolCall {
                    call_id: call.tool_call.id.into_string(),
                    name: call.tool_call.function.name,
                    arguments: call.tool_call.function.arguments,
                })
                .collect(),
        },
        AgentRunStep::Done(response) => ControlDirective::Done {
            output: response.output,
        },
    };
    checkpoint.state = serde_json::to_value(run).map_err(|error| error.to_string())?;
    Ok(directive)
}

pub fn accept_model(
    checkpoint: &mut ControlCheckpoint,
    response: ModelTurnResponse,
    advertised_tools: BTreeSet<String>,
) -> Result<(), String> {
    verify(checkpoint)?;
    if checkpoint.model_turn_count >= checkpoint.max_model_turns {
        return Err("assistant-budget-exhausted: model turn limit reached".into());
    }
    let mut run = restore(checkpoint)?;
    match run
        .model_response(conversion::model_turn(response, advertised_tools))
        .map_err(|error| error.to_string())?
    {
        ModelTurnOutcome::Continue { .. } => {}
        ModelTurnOutcome::NeedsResolution(_) => {
            return Err("tool-not-allowed: model requested an unavailable tool".into());
        }
        ModelTurnOutcome::TurnRetried => {}
    }
    checkpoint.transcript_cursor += 1;
    checkpoint.model_turn_count += 1;
    checkpoint.state = serde_json::to_value(run).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn accept_tools(
    checkpoint: &mut ControlCheckpoint,
    results: Vec<RuntimeToolReply>,
) -> Result<(), String> {
    verify(checkpoint)?;
    let added = u16::try_from(results.len())
        .map_err(|_| "assistant-budget-exhausted: tool call count overflow")?;
    if checkpoint.tool_call_count.saturating_add(added) > checkpoint.max_tool_calls {
        return Err("assistant-budget-exhausted: tool call limit reached".into());
    }
    let mut run = restore(checkpoint)?;
    let results = results
        .into_iter()
        .map(|tool_reply| {
            let call = ToolCallId::new(tool_reply.call_id)
                .ok_or("tool-schema-invalid: tool result call ID is empty")?;
            let mut content = vec![ToolResultContent::Json {
                value: tool_reply.result,
            }];
            for part in tool_reply.media {
                content.push(ToolResultContent::Image(media::to_image(part)?));
            }
            Ok(UserContent::ToolResult(ToolResult {
                call,
                provider: None,
                name: tool_reply.name,
                content,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    run.tool_results(results)
        .map_err(|error| error.to_string())?;
    checkpoint.tool_call_count += added;
    checkpoint.state = serde_json::to_value(run).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn requires_vision(checkpoint: &ControlCheckpoint) -> Result<bool, String> {
    verify(checkpoint)?;
    let run = restore(checkpoint)?;
    Ok(run.messages().iter().any(|message| match message {
        rig_agent::core::message::Message::User { content } => {
            content.iter().any(|part| match part {
                UserContent::Image(_) => true,
                UserContent::ToolResult(reply) => reply
                    .content
                    .iter()
                    .any(|part| matches!(part, ToolResultContent::Image(_))),
                _ => false,
            })
        }
        _ => false,
    }))
}

fn checkpoint(
    run: AgentRun,
    run_id: String,
    segment_id: String,
    catalog_digest: String,
    max_model_turns: u16,
    max_tool_calls: u16,
) -> Result<ControlCheckpoint, String> {
    Ok(ControlCheckpoint {
        schema_version: 1,
        runtime: RUNTIME.into(),
        runtime_version: RUNTIME_VERSION.into(),
        run_id,
        segment_id,
        current_turn_id: None,
        control_generation: 0,
        catalog_digest,
        transcript_cursor: 0,
        model_turn_count: 0,
        tool_call_count: 0,
        max_model_turns,
        max_tool_calls,
        state: serde_json::to_value(run).map_err(|error| error.to_string())?,
    })
}

fn restore(checkpoint: &ControlCheckpoint) -> Result<AgentRun, String> {
    serde_json::from_value(checkpoint.state.clone()).map_err(|error| error.to_string())
}

fn verify(checkpoint: &ControlCheckpoint) -> Result<(), String> {
    if checkpoint.schema_version != 1
        || checkpoint.runtime != RUNTIME
        || checkpoint.runtime_version != RUNTIME_VERSION
    {
        return Err("checkpoint-version-blocked: Agent Runtime checkpoint is incompatible".into());
    }
    Ok(())
}
