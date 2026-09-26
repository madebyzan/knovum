use sqlx::SqlitePool;

use crate::spaces::{
    assistant::{tools::ToolCatalog, AssistantMode, StartAssistantTaskRequest},
    authorization::{AuthorizationContext, Principal, PrincipalKind},
    db,
};

pub(super) struct TaskIds {
    pub task_id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub grant_id: String,
}

impl TaskIds {
    pub fn new(request: &StartAssistantTaskRequest) -> Self {
        Self {
            task_id: uuid::Uuid::new_v4().to_string(),
            run_id: uuid::Uuid::new_v4().to_string(),
            conversation_id: request
                .conversation_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            branch_id: request
                .branch_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            grant_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

pub(super) fn grant(
    request: &StartAssistantTaskRequest,
    ids: &TaskIds,
    catalog: &ToolCatalog,
    now: i64,
) -> AuthorizationContext {
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::Assistant,
            id: ids.task_id.clone(),
        },
        grant_id: ids.grant_id.clone(),
        home_space_id: request.space_id.clone(),
        read_space_ids: vec![request.space_id.clone()],
        write_space_ids: matches!(request.mode, AssistantMode::Execute)
            .then(|| vec![request.space_id.clone()])
            .unwrap_or_default(),
        action_ids: catalog
            .by_wire_name
            .values()
            .map(|definition| definition.id.to_owned())
            .collect(),
        valid_until: now + 2 * 60 * 60 * 1_000,
    }
}

pub(super) fn validate(request: &StartAssistantTaskRequest) -> Result<(), String> {
    if request.space_id.trim().is_empty() || request.prompt.trim().is_empty() {
        return Err("Assistant Space and prompt are required".into());
    }
    if !(1..=32).contains(&request.max_turns) {
        return Err("Assistant maxTurns must be between 1 and 32".into());
    }
    if !(1..=128).contains(&request.max_tool_calls) {
        return Err("Assistant maxToolCalls must be between 1 and 128".into());
    }
    if !(1_000..=1_000_000).contains(&request.max_total_tokens) {
        return Err("Assistant maxTotalTokens must be between 1000 and 1000000".into());
    }
    if !(30..=7_200).contains(&request.max_duration_seconds) {
        return Err("Assistant maxDurationSeconds must be between 30 and 7200".into());
    }
    Ok(())
}

pub(super) async fn ensure_within_duration(pool: &SqlitePool, run_id: &str) -> Result<(), String> {
    remaining_duration(pool, run_id).await.map(|_| ())
}

pub(super) async fn remaining_duration(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<std::time::Duration, String> {
    let remaining: Option<i64> = sqlx::query_scalar(
        "SELECT json_extract(budget,'$.maxDurationSeconds')*1000-($1-created_at) FROM spaces_runs WHERE id=$2",
    ).bind(crate::clock::unix_time_millis()?).bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    remaining
        .filter(|value| *value > 0)
        .map(|millis| std::time::Duration::from_millis(millis as u64))
        .ok_or_else(|| "budget-exhausted: Assistant duration limit was exceeded".into())
}

pub(super) async fn ensure_active(pool: &SqlitePool, run_id: &str) -> Result<(), String> {
    let active: Option<i64> = sqlx::query_scalar("SELECT 1 FROM spaces_runs WHERE id=$1 AND state='running' AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND completed_at IS NULL")
        .bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    active
        .ok_or_else(|| "assistant-task-inactive: task was cancelled, paused, or completed".into())
        .map(|_| ())
}

pub(super) fn conversation_title(prompt: &str) -> String {
    prompt.chars().take(80).collect()
}
