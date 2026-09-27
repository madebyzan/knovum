use super::super::{
    agent_task, contract, task, AssistantPermissionOption, AssistantTaskAccepted,
    AssistantTaskUpdate, SubmitAssistantTaskRequest,
};
use crate::command_error::CommandError;

#[derive(serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ActiveAssistantRun {
    pub run: crate::spaces::runs::RunObservation,
    pub conversation_id: String,
    pub branch_id: String,
    pub title: String,
}

#[tauri::command]
pub async fn list_active_assistant_runs(
    app: tauri::AppHandle,
    space_id: Option<String>,
) -> Result<Vec<ActiveAssistantRun>, CommandError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let identities: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT r.id,t.conversation_id,t.branch_id,c.title FROM spaces_runs r
         JOIN spaces_assistant_tasks t ON t.run_id=r.id
         JOIN spaces_assistant_conversations c ON c.id=t.conversation_id
         WHERE ($1 IS NULL OR r.home_space_id=$1) AND r.completed_at IS NULL
         UNION ALL
         SELECT r.id,e.assistant_conversation_id,e.branch_id,c.title FROM spaces_runs r
         JOIN spaces_external_agent_conversations e ON e.run_id=r.id
         JOIN spaces_assistant_conversations c ON c.id=e.assistant_conversation_id
         WHERE ($1 IS NULL OR r.home_space_id=$1) AND r.completed_at IS NULL",
    )
    .bind(space_id.as_deref())
    .fetch_all(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    let mut runs = Vec::with_capacity(identities.len());
    for (run_id, conversation_id, branch_id, title) in identities {
        runs.push(ActiveAssistantRun {
            run: crate::spaces::runs::load_run(pool, &run_id).await?,
            conversation_id,
            branch_id,
            title,
        });
    }
    runs.sort_by_key(|entry| std::cmp::Reverse(entry.run.created_at));
    Ok(runs)
}

#[tauri::command]
pub async fn pending_assistant_permission(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Option<AssistantTaskUpdate>, CommandError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let session_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_external_agent_conversations
         WHERE run_id=$1 AND state='connected'",
    )
    .bind(&run_id)
    .fetch_optional(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    let Some(session_id) = session_id else {
        return Ok(None);
    };
    let pending = crate::external_agents::permissions::pending_for_session(&session_id)?;
    Ok(pending
        .into_iter()
        .next()
        .map(|request| AssistantTaskUpdate::PermissionRequested {
            session_id,
            request_id: request.request_id,
            title: request.title,
            options: request
                .options
                .into_iter()
                .map(|option| AssistantPermissionOption {
                    id: option.id,
                    label: option.label,
                    kind: option.kind.to_owned(),
                })
                .collect(),
        }))
}

#[tauri::command]
pub async fn get_assistant_task_error(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Option<String>, CommandError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let model: Option<Option<String>> =
        sqlx::query_scalar("SELECT error FROM spaces_assistant_tasks WHERE run_id=$1")
            .bind(&run_id)
            .fetch_optional(pool)
            .await
            .map_err(crate::spaces::db::sql_error)?;
    if let Some(error) = model {
        return Ok(error);
    }
    let agent: Option<Option<String>> =
        sqlx::query_scalar("SELECT error FROM spaces_external_agent_conversations WHERE run_id=$1")
            .bind(run_id)
            .fetch_optional(pool)
            .await
            .map_err(crate::spaces::db::sql_error)?;
    Ok(agent.flatten())
}

#[tauri::command]
pub async fn start_assistant_task(
    app: tauri::AppHandle,
    request: SubmitAssistantTaskRequest,
    updates: tauri::ipc::Channel<AssistantTaskUpdate>,
) -> Result<AssistantTaskAccepted, CommandError> {
    let accepted = match request.target.clone() {
        contract::AssistantExecutionTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } => {
            task::start(
                app,
                request.into_model(crate::ai::models::ExecutionTargetSelection::Model {
                    connection_id,
                    model_id,
                    temperature,
                }),
                updates,
            )
            .await
        }
        contract::AssistantExecutionTargetSelection::Agent { runtime_kind } => {
            agent_task::start(app, request, runtime_kind, updates).await
        }
    };
    Ok(accepted?)
}

pub(crate) fn cancel_task(run_id: &str) -> Result<(), String> {
    task::cancel(run_id)
}
