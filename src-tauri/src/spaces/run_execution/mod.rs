use std::{future::Future, pin::Pin};

use serde_json::Value;
use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::{
    app_error::AppError,
    spaces::runs::{AttemptClaim, SchedulerHooks, StepOutcome},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Route {
    Speech,
    VoiceCandidate,
    SourceSeparation,
    Translation,
    Captured,
    Action,
    Unsupported,
}

pub(super) fn hooks() -> SchedulerHooks {
    SchedulerHooks {
        execute_step: execute_hook,
        after_attempt: after_attempt_hook,
        after_controls: after_controls_hook,
    }
}

fn execute_hook<'a>(
    app: &'a AppHandle,
    pool: &'a SqlitePool,
    claim: &'a AttemptClaim,
) -> Pin<Box<dyn Future<Output = Result<StepOutcome, AppError>> + Send + 'a>> {
    Box::pin(execute_step(app, pool, claim))
}

fn after_attempt_hook<'a>(
    app: &'a AppHandle,
    pool: &'a SqlitePool,
    claim: &'a AttemptClaim,
) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(
        crate::spaces::speech::voice_candidates::cleanup_terminal_snapshot(
            app,
            pool,
            &claim.run_id,
        ),
    )
}

fn after_controls_hook<'a>(
    app: &'a AppHandle,
    pool: &'a SqlitePool,
    controls_changed: bool,
) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        if controls_changed {
            crate::spaces::speech::voice_candidates::reconcile_terminal_snapshots(app, pool)
                .await?;
        }
        Ok(())
    })
}

async fn execute_step(
    app: &AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    match route(&claim.node_kind, &claim.configuration) {
        Route::Speech => crate::spaces::speech::run_execution::execute(app, pool, claim).await,
        Route::VoiceCandidate => {
            crate::spaces::speech::voice_candidates::execute(app, pool, claim).await
        }
        Route::SourceSeparation => {
            crate::spaces::speech::source_separation::execute(app, pool, claim).await
        }
        Route::Translation => crate::spaces::speech::translation::execute(app, pool, claim).await,
        Route::Captured => crate::spaces::workflows::execute_captured_step(pool, claim).await,
        Route::Action => execute_action(app, claim).await,
        Route::Unsupported => Err(AppError::new(
            "run_step_invalid",
            format!("Unsupported execution node {}", claim.node_kind),
        )),
    }
}

async fn execute_action(app: &AppHandle, claim: &AttemptClaim) -> Result<StepOutcome, AppError> {
    let action_id = required_str(&claim.resolved_input, "actionId")?;
    let parameters = claim
        .resolved_input
        .get("parameters")
        .ok_or("action-input-invalid: parameters are required")?;
    let inputs = claim.resolved_input.get("inputs").unwrap_or(&Value::Null);
    let output =
        crate::spaces::action_execution::execute_durable(app, action_id, claim, parameters, inputs)
            .await?;
    Ok(StepOutcome::Completed {
        output,
        selected_branch: None,
    })
}

fn route(node_kind: &str, configuration: &Value) -> Route {
    match node_kind {
        "speech_voice_candidate" => Route::VoiceCandidate,
        "speech_source_separation" => Route::SourceSeparation,
        crate::spaces::speech::translation::NODE_KIND => Route::Translation,
        "speech_segment"
        | "speech_chapter_pcm"
        | "speech_loudness_measurement"
        | "speech_audiobook_encode"
        | "speech_assembly" => Route::Speech,
        "captured_action" => Route::Captured,
        "action"
            if configuration
                .as_object()
                .is_some_and(serde_json::Map::is_empty) =>
        {
            Route::Action
        }
        _ => Route::Unsupported,
    }
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("action-input-invalid: {key} is required"))
}

#[cfg(test)]
mod tests;
