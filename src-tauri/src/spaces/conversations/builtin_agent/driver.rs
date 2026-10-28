use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use serde_json::{json, Value};
use tauri::Manager;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::{
    agent_runtime::{
        process::Process,
        protocol::{
            control::{
                AgentControl, AgentControlOperation, AgentExecute, AgentExecutionObservation,
                AgentExecutionState,
            },
            messages::{AgentInputMessage, AgentProjection, AgentProjectionRequest},
            tools::AgentToolDefinition,
        },
    },
    spaces::{
        conversations::execution::{binding, gateway::Gateway, ledger, projection},
        db,
    },
};

pub(super) enum DriveOutcome {
    Completed,
    Paused,
    Waiting,
}

pub(super) async fn drive(
    host: Gateway,
    messages: Vec<AgentInputMessage>,
    resume: Option<Value>,
    cancellation: CancellationToken,
) -> Result<DriveOutcome, String> {
    static EXECUTION: OnceLock<Semaphore> = OnceLock::new();
    let deadline = tokio::time::Instant::now()
        + super::identity::remaining_duration(&host.pool, &host.ids.run_id).await?;
    let _permit = tokio::time::timeout_at(
        deadline,
        EXECUTION.get_or_init(|| Semaphore::new(1)).acquire(),
    )
    .await
    .map_err(|_| "builtin-agent-duration-budget-exhausted")?
    .map_err(|_| "builtin-agent-process-admission-closed")?;
    if cancellation.is_cancelled() {
        return Err("builtin-agent-cancelled".into());
    }
    let checkpoint_path = host
        .app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("agent-runtime/checkpoints.sqlite");
    let model = binding::resolve(&host.target, &host.profile).await?;
    let space_snapshot: String = sqlx::query_scalar(
        "SELECT json_extract(input_snapshot,'$.spaceContext') FROM spaces_runs WHERE id=$1",
    )
    .bind(&host.ids.run_id)
    .fetch_one(&host.pool)
    .await
    .map_err(db::sql_error)?;
    let workspace = crate::spaces::agent_artifacts::workspace::builtin_agent_directory(
        &host.app,
        &host.request.space_id,
        &host.ids.conversation_id,
    )
    .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&workspace).map_err(|error| error.to_string())?;
    let execute = AgentExecute {
        identity: host.session.identity(&host.ids.run_id),
        model,
        system_prompt: crate::spaces::conversations::prompt_policy::builtin_agent(&space_snapshot),
        tools: host
            .catalog
            .tools
            .iter()
            .map(|tool| AgentToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
                read_only: host.catalog.binding_by_wire_name[&tool.name].read_only,
            })
            .collect(),
        messages,
        input_id: host.session.input_id.clone(),
        after_checkpoint_id: host.session.projection_checkpoint_id.clone(),
        resume,
        skills: host.extensions.files.clone(),
        skill_tool_bindings: host.extensions.skill_tool_bindings.clone(),
        workspace: workspace
            .to_str()
            .ok_or("builtin-agent-workspace-not-unicode")?
            .to_owned(),
    };
    let host = Arc::new(host);
    let mut process = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err("builtin-agent-cancelled".into()),
        started = tokio::time::timeout_at(
            deadline,
            Process::start(&host.app, &checkpoint_path, host.clone()),
        ) => started.map_err(|_| "builtin-agent-duration-budget-exhausted")??,
    };
    let outcome = tokio::time::timeout_at(
        deadline,
        async {
            tokio::select! {
                biased;
                _ = cancellation.cancelled() => {
                    if is_pause(&host).await? {
                        paused(&host).await?;
                        return Ok(DriveOutcome::Paused);
                    }
                    return Err("builtin-agent-cancelled".into());
                }
                retention = crate::spaces::conversations::execution::retention::prune(&host.pool, &process.peer) => retention?,
            }
            execute_loop(&process, &host, execute, cancellation.clone()).await
        },
    )
    .await
    .map_err(|_| "builtin-agent-duration-budget-exhausted".to_owned())
    .and_then(std::convert::identity);
    process.terminate().await;
    outcome
}

async fn execute_loop(
    process: &Process,
    host: &Gateway,
    mut execute: AgentExecute,
    cancellation: CancellationToken,
) -> Result<DriveOutcome, String> {
    loop {
        let outcome = {
            let execution = process
                .peer
                .request::<AgentExecutionObservation>("agentRuntime.execute", &execute);
            tokio::pin!(execution);
            tokio::select! {
                outcome = &mut execution => outcome,
                _ = cancellation.cancelled() => {
                    let pause = is_pause(host).await?;
                    process.peer.notify("agentRuntime.control", &AgentControl { identity: execute.identity.clone(), operation: if pause { AgentControlOperation::Pause } else { AgentControlOperation::Cancel } }).await?;
                    let settling = tokio::time::timeout(Duration::from_secs(if pause { 65 } else { 5 }), &mut execution).await;
                    if !pause { return Err("builtin-agent-cancelled".into()); }
                    settling.map_err(|_| "builtin-agent-pause-not-settled".to_owned())?
                }
            }
        };
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(_) if cancellation.is_cancelled() && is_pause(host).await? => {
                synchronize_projection(process, host).await?;
                ledger::require_reconciled(&host.pool, &host.ids.task_id).await?;
                paused(host).await?;
                return Ok(DriveOutcome::Paused);
            }
            Err(error) => return Err(error),
        };
        if outcome.identity.run_id != execute.identity.run_id
            || outcome.identity.thread_id != execute.identity.thread_id
        {
            return Err("builtin-agent-execution-retention-mismatch".into());
        }
        if matches!(outcome.state, AgentExecutionState::Completed) {
            if is_pause(host).await? {
                paused(host).await?;
                return Ok(DriveOutcome::Paused);
            }
            return Ok(DriveOutcome::Completed);
        }
        if is_pause(host).await?
            || outcome
                .interrupts
                .iter()
                .any(|value| value["type"] == "pause")
        {
            paused(host).await?;
            return Ok(DriveOutcome::Paused);
        }
        if let Some(interrupt) = outcome
            .interrupts
            .iter()
            .find(|value| value["type"] == "tool_intervention")
        {
            let reason = interrupt["reason"]
                .as_str()
                .ok_or("agent-tool-intervention-reason-missing")?;
            set_state(host, "waiting", Some(reason)).await?;
            return Ok(DriveOutcome::Waiting);
        }
        let children = outcome
            .interrupts
            .iter()
            .map(|value| {
                if value["type"] != "child_run" {
                    return Err("builtin-agent-interrupt-unsupported".to_owned());
                }
                value["runId"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "builtin-agent-child-identity-missing".into())
            })
            .collect::<Result<Vec<_>, String>>()?;
        set_waiting(host).await?;
        for child in children {
            tokio::select! {
                outcome = super::tool_execution::child::wait(&host.pool, &child) => outcome?,
                _ = cancellation.cancelled() => {
                    if is_pause(host).await? { paused(host).await?; return Ok(DriveOutcome::Paused); }
                    return Err("builtin-agent-cancelled".into());
                }
            }
        }
        set_running(host).await?;
        execute.after_checkpoint_id = Some(outcome.checkpoint_id);
        execute.messages.clear();
        execute.resume = Some(json!(true));
    }
}

async fn synchronize_projection(process: &Process, host: &Gateway) -> Result<(), String> {
    let projection: AgentProjection = process
        .peer
        .request(
            "agentRuntime.projection",
            &AgentProjectionRequest {
                thread_id: host.session.thread_id.clone(),
                after_checkpoint_id: host.session.projection_checkpoint_id.clone(),
            },
        )
        .await?;
    projection::commit(&host.pool, &host.ids, projection).await?;
    Ok(())
}

async fn is_pause(host: &Gateway) -> Result<bool, String> {
    let state: (Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT pause_requested_at,cancel_requested_at FROM spaces_runs WHERE id=$1",
    )
    .bind(&host.ids.run_id)
    .fetch_one(&host.pool)
    .await
    .map_err(db::sql_error)?;
    Ok(state.0.is_some() && state.1.is_none())
}

async fn paused(host: &Gateway) -> Result<(), String> {
    let mut tx = db::begin_write(&host.pool).await?;
    let updated = sqlx::query("UPDATE spaces_runs SET state='paused',wait_reason=NULL,state_version=state_version+1,updated_at=$1
        WHERE id=$2 AND pause_requested_at IS NOT NULL AND cancel_requested_at IS NULL AND completed_at IS NULL")
        .bind(crate::foundation::clock::unix_time_millis()?).bind(&host.ids.run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(updated, "builtin-agent-pause-inactive")?;
    crate::spaces::runs::append_event(&mut tx, &host.ids.run_id, None, None, crate::spaces::runs::RunEventKind::BuiltinAgentPausedCheckpoint,
        &json!({"threadId":host.session.thread_id,"graphVersion":crate::agent_runtime::protocol::GRAPH_VERSION}), crate::foundation::clock::unix_time_millis()?).await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

async fn set_waiting(host: &Gateway) -> Result<(), String> {
    set_state(host, "waiting", Some("executor")).await
}
async fn set_running(host: &Gateway) -> Result<(), String> {
    set_state(host, "running", None).await
}
async fn set_state(host: &Gateway, state: &str, reason: Option<&str>) -> Result<(), String> {
    let changed = sqlx::query("UPDATE spaces_runs SET state=$1,wait_reason=$2,state_version=state_version+1,updated_at=$3
        WHERE id=$4 AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND completed_at IS NULL")
        .bind(state).bind(reason).bind(crate::foundation::clock::unix_time_millis()?).bind(&host.ids.run_id).execute(&host.pool).await.map_err(db::sql_error)?;
    db::require_one(changed, "builtin-agent-execution-inactive")?;
    crate::spaces::runs::announce_change();
    Ok(())
}
