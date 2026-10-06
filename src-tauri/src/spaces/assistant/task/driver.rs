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
                AssistantControl, AssistantControlOperation, AssistantExecute,
                AssistantExecutionResult, AssistantExecutionState,
            },
            messages::{AssistantInputMessage, AssistantProjection, AssistantProjectionRequest},
            tools::AssistantToolDefinition,
        },
    },
    local_resources::runtimes::python::RuntimeProfile,
    spaces::{
        assistant::{
            execution::{binding, gateway::Gateway, ledger, projection},
            AssistantMode,
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
    messages: Vec<AssistantInputMessage>,
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
    .map_err(|_| "assistant-duration-budget-exhausted")?
    .map_err(|_| "assistant-process-admission-closed")?;
    tokio::time::timeout_at(
        deadline,
        crate::local_resources::preparation::prepare_python_for_job(
            &host.app,
            RuntimeProfile::Assistant,
            &format!("assistant-runtime:{}", host.ids.run_id),
            "Prepare built-in Assistant runtime",
            &cancellation,
        ),
    )
    .await
    .map_err(|_| "assistant-duration-budget-exhausted")??;
    let checkpoint_path = host
        .app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("assistant/checkpoints.sqlite");
    let model = binding::resolve(&host.target, &host.profile).await?;
    let memory_snapshot: String = sqlx::query_scalar(
        "SELECT json_extract(input_snapshot,'$.spaceContext') FROM spaces_runs WHERE id=$1",
    )
    .bind(&host.ids.run_id)
    .fetch_one(&host.pool)
    .await
    .map_err(db::sql_error)?;
    let memory = std::collections::BTreeMap::from([("/space.json".into(), memory_snapshot)]);
    let execute = AssistantExecute {
        identity: host.session.identity(&host.ids.run_id), model,
        system_prompt: format!("You are Knovum's built-in knowledge assistant. Mode: {}. {} Use provided Actions for business changes. The user authorizes execution of the available Actions; do not ask for redundant confirmation. Publish only through Knovum Actions and never claim success without a completed Action receipt.", host.request.mode.as_str(), crate::spaces::assistant::prompt_policy::SPACE_READING),
        tools: host.catalog.tools.iter().map(|tool| AssistantToolDefinition {
            name: tool.name.clone(), description: tool.description.clone(), parameters: tool.input_schema.clone(),
            read_only: host.catalog.by_wire_name[&tool.name].impact == crate::spaces::actions::SpaceActionImpact::ReadOnly,
        }).collect(), messages, input_id: host.session.input_id.clone(), after_checkpoint_id: host.session.projection_checkpoint_id.clone(), resume,
        skills: crate::spaces::assistant::skills::snapshots(),
        memory,
        plan: !matches!(host.request.mode, AssistantMode::Ask), research: true,
    };
    let host = Arc::new(host);
    let mut process = tokio::time::timeout_at(
        deadline,
        Process::start(&host.app, &checkpoint_path, host.clone()),
    )
    .await
    .map_err(|_| "assistant-duration-budget-exhausted")??;
    crate::spaces::assistant::execution::retention::prune(&host.pool, &process.peer).await?;
    let outcome = tokio::time::timeout_at(
        deadline,
        execute_loop(&process, &host, execute, cancellation.clone()),
    )
    .await
    .map_err(|_| "assistant-duration-budget-exhausted".to_owned())
    .and_then(std::convert::identity);
    process.terminate().await;
    outcome
}

async fn execute_loop(
    process: &Process,
    host: &Gateway,
    mut execute: AssistantExecute,
    cancellation: CancellationToken,
) -> Result<DriveOutcome, String> {
    loop {
        let outcome = {
            let execution = process
                .peer
                .request::<AssistantExecutionResult>("assistant.execute", &execute);
            tokio::pin!(execution);
            tokio::select! {
                outcome = &mut execution => outcome,
                _ = cancellation.cancelled() => {
                    let pause = is_pause(host).await?;
                    process.peer.notify("assistant.control", &AssistantControl { identity: execute.identity.clone(), operation: if pause { AssistantControlOperation::Pause } else { AssistantControlOperation::Cancel } }).await?;
                    let settling = tokio::time::timeout(Duration::from_secs(if pause { 65 } else { 5 }), &mut execution).await;
                    if !pause { return Err("assistant-cancelled".into()); }
                    settling.map_err(|_| "assistant-pause-not-settled".to_owned())?
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
            return Err("assistant-execution-result-mismatch".into());
        }
        if matches!(outcome.state, AssistantExecutionState::Completed) {
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
                    return Err("assistant-interrupt-unsupported".to_owned());
                }
                value["runId"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "assistant-child-identity-missing".into())
            })
            .collect::<Result<Vec<_>, String>>()?;
        set_waiting(host).await?;
        for child in children {
            tokio::select! {
                outcome = super::tool_execution::child::wait(&host.pool, &child) => outcome?,
                _ = cancellation.cancelled() => {
                    if is_pause(host).await? { paused(host).await?; return Ok(DriveOutcome::Paused); }
                    return Err("assistant-cancelled".into());
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
    let projection: AssistantProjection = process
        .peer
        .request(
            "assistant.projection",
            &AssistantProjectionRequest {
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
    db::require_one(updated, "assistant-pause-inactive")?;
    crate::spaces::runs::append_event(&mut tx, &host.ids.run_id, None, None, crate::spaces::runs::RunEventKind::AssistantPausedCheckpoint,
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
    db::require_one(changed, "assistant-execution-inactive")?;
    crate::spaces::runs::announce_change();
    Ok(())
}
