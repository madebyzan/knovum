use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use ts_rs::TS;

use crate::spaces::db;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum RunEventKind {
    AssistantPausedCheckpoint,
    AssistantToolStarted,
    AssistantToolFinished,
    AttemptClaimed,
    ExternalAgentMessage,
    ExternalAgentToolStarted,
    ExternalAgentToolUpdated,
    ExternalOutcomeReconciled,
    RunBudgetExhausted,
    RunQueued,
    RunRetryCreated,
    StepFailed,
    StepRetryScheduled,
}

impl RunEventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AssistantPausedCheckpoint => "assistant_paused_checkpoint",
            Self::AssistantToolStarted => "assistant_tool_started",
            Self::AssistantToolFinished => "assistant_tool_finished",
            Self::AttemptClaimed => "attempt_claimed",
            Self::ExternalAgentMessage => "external_agent_message",
            Self::ExternalAgentToolStarted => "external_agent_tool_started",
            Self::ExternalAgentToolUpdated => "external_agent_tool_updated",
            Self::ExternalOutcomeReconciled => "external_outcome_reconciled",
            Self::RunBudgetExhausted => "run_budget_exhausted",
            Self::RunQueued => "run_queued",
            Self::RunRetryCreated => "run_retry_created",
            Self::StepFailed => "step_failed",
            Self::StepRetryScheduled => "step_retry_scheduled",
        }
    }
}

pub(crate) async fn append_event(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    step_instance_id: Option<&str>,
    attempt_id: Option<&str>,
    kind: RunEventKind,
    payload: &serde_json::Value,
    now: i64,
) -> Result<i64, String> {
    let payload = serde_json::to_string(payload).map_err(db::json_error)?;
    let event_write = sqlx::query(
        "INSERT INTO spaces_run_events(run_id,step_instance_id,attempt_id,kind,payload,created_at)
         VALUES($1,$2,$3,$4,$5,$6)",
    )
    .bind(run_id)
    .bind(step_instance_id)
    .bind(attempt_id)
    .bind(kind.as_str())
    .bind(payload)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(event_write.last_insert_rowid())
}
