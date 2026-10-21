use sqlx::SqlitePool;

use crate::spaces::{
    authorization::{AuthorizationContext, Principal, PrincipalKind, RunScope},
    conversations::{tools::ToolCatalog, StartBuiltinAgentTaskRequest},
    db,
};

#[derive(Clone)]
pub(in crate::spaces::conversations) struct TaskIds {
    pub task_id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub grant_id: String,
}

impl TaskIds {
    pub(in crate::spaces::conversations) fn input_id(&self) -> String {
        format!("input:{}", self.task_id)
    }

    pub fn new(request: &StartBuiltinAgentTaskRequest) -> Self {
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

pub(in crate::spaces::conversations) fn grant(
    request: &StartBuiltinAgentTaskRequest,
    task_ids: &TaskIds,
    catalog: &ToolCatalog,
    now: i64,
) -> AuthorizationContext {
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::BuiltinAgent,
            id: task_ids.task_id.clone(),
        },
        grant_id: task_ids.grant_id.clone(),
        scope: RunScope::Space {
            space_id: request.space_id.clone(),
        },
        read_space_ids: vec![request.space_id.clone()],
        write_space_ids: vec![request.space_id.clone()],
        action_ids: catalog
            .binding_by_wire_name
            .values()
            .filter_map(|binding| match &binding.source {
                crate::extensions::contracts::AgentToolBinding::SpaceAction {
                    action_id, ..
                } => Some(action_id.clone()),
                _ => None,
            })
            .collect(),
        valid_until: now + 2 * 60 * 60 * 1_000,
    }
}

pub(in crate::spaces::conversations) fn validate(
    request: &StartBuiltinAgentTaskRequest,
) -> Result<(), String> {
    if request.space_id.trim().is_empty() || request.prompt.trim().is_empty() {
        return Err("Builtin Agent Space and prompt are required".into());
    }
    if !(1..=32).contains(&request.max_turns) {
        return Err("Builtin Agent maxTurns must be between 1 and 32".into());
    }
    if !(1..=128).contains(&request.max_tool_calls) {
        return Err("Builtin Agent maxToolCalls must be between 1 and 128".into());
    }
    if !(30..=7_200).contains(&request.max_duration_seconds) {
        return Err("Builtin Agent maxDurationSeconds must be between 30 and 7200".into());
    }
    Ok(())
}

pub(in crate::spaces::conversations) async fn remaining_duration(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<std::time::Duration, String> {
    let remaining: Option<i64> = sqlx::query_scalar(
        "SELECT json_extract(budget,'$.maxDurationSeconds')*1000-($1-created_at) FROM spaces_runs WHERE id=$2",
    ).bind(crate::foundation::clock::unix_time_millis()?).bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    remaining
        .filter(|value| *value > 0)
        .map(|millis| std::time::Duration::from_millis(millis as u64))
        .ok_or_else(|| "budget-exhausted: Builtin Agent duration limit was exceeded".into())
}

pub(in crate::spaces::conversations) async fn ensure_active(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    let active: Option<i64> = sqlx::query_scalar("SELECT 1 FROM spaces_runs WHERE id=$1 AND state IN ('running','waiting') AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND completed_at IS NULL")
        .bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    active
        .ok_or_else(|| {
            "builtin-agent-task-inactive: task was cancelled, paused, or completed".into()
        })
        .map(|_| ())
}

pub(in crate::spaces::conversations) fn conversation_title(prompt: &str) -> String {
    prompt.chars().take(80).collect()
}
