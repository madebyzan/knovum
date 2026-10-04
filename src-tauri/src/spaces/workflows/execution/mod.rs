use crate::app_error::AppError;
mod action;
mod shared;

use serde_json::json;
use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::spaces::runs::{AttemptClaim, StepOutcome};

pub(crate) async fn execute_step(
    app: &AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    if matches!(
        claim.node_kind.as_str(),
        "speech_segment"
            | "speech_chapter_pcm"
            | "speech_loudness_measurement"
            | "speech_audiobook_encode"
            | "speech_assembly"
            | "speech_voice_candidate"
            | "speech_source_separation"
    ) {
        if claim.node_kind == "speech_voice_candidate" {
            return crate::spaces::speech::voice_candidates::execute(app, pool, claim).await;
        }
        if claim.node_kind == "speech_source_separation" {
            return crate::spaces::speech::source_separation::execute(app, pool, claim).await;
        }
        return crate::spaces::speech::run_execution::execute(app, pool, claim).await;
    }
    if crate::spaces::speech::translation::is_run_step(&claim.node_kind) {
        return crate::spaces::speech::translation::execute(app, pool, claim).await;
    }
    if claim.node_kind == "captured_action" {
        return super::capture::execute_step(pool, claim).await;
    }
    if claim.node_kind == "action" && claim.configuration == json!({}) {
        return action::execute_run(app, claim, &claim.resolved_input).await;
    }
    Err(AppError::new(
        "workflow_step_invalid",
        format!("Unsupported execution node {}", claim.node_kind),
    ))
}

pub(crate) async fn invoke_captured(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: serde_json::Value,
    input: serde_json::Value,
) -> Result<StepOutcome, AppError> {
    action::execute_invocation(pool, claim, action_id, action_version, parameters, input).await
}
