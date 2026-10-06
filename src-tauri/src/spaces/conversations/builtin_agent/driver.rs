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
                AgentControl, AgentControlOperation, AgentExecute, AgentExecutionResult,
                AgentExecutionState,
            },
            messages::{AgentInputMessage, AgentProjection, AgentProjectionRequest},
            tools::AgentToolDefinition,
        },
    },
    local_resources::runtimes::python::RuntimeProfile,
    spaces::{
        conversations::{
            execution::{binding, gateway::Gateway, ledger, projection},
            ConversationMode,
        },
        db,
    },
};

pub(super) enum DriveOutcome {
    Completed,
    Paused,
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
    tokio::time::timeout_at(
        deadline,
        crate::local_resources::preparation::prepare_python_for_job(
            &host.app,
            RuntimeProfile::Agent,
            &format!("builtin-agent-runtime:{}", host.ids.run_id),
            "Prepare Agent Runtime",
            &cancellation,
        ),
    )
    .await
    .map_err(|_| "builtin-agent-duration-budget-exhausted")??;
    let checkpoint_path = host
        .app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("agent-runtime/checkpoints.sqlite");
    let model = binding::resolve(&host.target, &host.profile).await?;
    let memory_snapshot: String = sqlx::query_scalar(
        "SELECT json_extract(input_snapshot,'$.spaceContext') FROM spaces_runs WHERE id=$1",
    )
    .bind(&host.ids.run_id)
    .fetch_one(&host.pool)
    .await
    .map_err(db::sql_error)?;
    let memory = std::collections::BTreeMap::from([("/space.json".into(), memory_snapshot)]);
    let execute = AgentExecute {
        identity: host.session.identity(&host.ids.run_id), model,
        system_prompt: format!("You are Knovum's built-in knowledge agentRuntime. Mode: {}. {} Use provided Actions for business changes. The user authorizes execution of the available Actions; do not ask for redundant confirmation. Publish only through Knovum Actions and never claim success without a completed Action receipt.", host.request.mode.as_str(), crate::spaces::conversations::prompt_policy::SPACE_READING),
        tools: host.catalog.tools.iter().map(|tool| AgentToolDefinition {
            name: tool.name.clone(), description: tool.description.clone(), parameters: tool.input_schema.clone(),
            read_only: host.catalog.by_wire_name[&tool.name].impact == crate::spaces::actions::SpaceActionImpact::ReadOnly,
        }).collect(), messages, input_id: host.session.input_id.clone(), after_checkpoint_id: host.session.projection_checkpoint_id.clone(), resume,
        skills: crate::spaces::conversations::skills::snapshots(),
        memory,
        plan: !matches!(host.request.mode, ConversationMode::Ask), research: true,
    };
    let host = Arc::new(host);
    let mut process = tokio::time::timeout_at(
        deadline,
        Process::start(&host.app, &checkpoint_path, host.clone()),
    )
    .await
    .map_err(|_| "builtin-agent-duration-budget-exhausted")??;
    crate::spaces::conversations::execution::retention::prune(&host.pool, &process.peer).await?;
    let outcome = tokio::time::timeout_at(
        deadline,
        execute_loop(&process, &host, execute, cancellation.clone()),
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
                .request::<AgentExecutionResult>("agentRuntime.execute", &execute);
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
            return Err("builtin-agent-execution-result-mismatch".into());
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
        .bind(crate::clock::unix_time_millis()?).bind(&host.ids.run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(updated, "builtin-agent-pause-inactive")?;
    crate::spaces::runs::append_event(&mut tx, &host.ids.run_id, None, None, crate::spaces::runs::RunEventKind::BuiltinAgentPausedCheckpoint,
        &json!({"threadId":host.session.thread_id,"graphVersion":crate::agent_runtime::protocol::GRAPH_VERSION}), crate::clock::unix_time_millis()?).await?;
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
        .bind(state).bind(reason).bind(crate::clock::unix_time_millis()?).bind(&host.ids.run_id).execute(&host.pool).await.map_err(db::sql_error)?;
    db::require_one(changed, "builtin-agent-execution-inactive")?;
    crate::spaces::runs::announce_change();
    Ok(())
}
