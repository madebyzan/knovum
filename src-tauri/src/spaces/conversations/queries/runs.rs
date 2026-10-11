use crate::foundation::app_error::AppError;

#[derive(serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ActiveConversationRun {
    pub run: crate::spaces::runs::RunObservation,
    pub conversation_id: String,
    pub branch_id: String,
    pub title: String,
}

#[tauri::command]
pub async fn list_active_conversation_runs(
    app: tauri::AppHandle,
    space_id: Option<String>,
) -> Result<Vec<ActiveConversationRun>, AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let identities: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT r.id,t.conversation_id,t.branch_id,c.title FROM spaces_runs r
         JOIN spaces_builtin_agent_tasks t ON t.run_id=r.id
         JOIN spaces_conversations c ON c.id=t.conversation_id
         WHERE r.scope_kind='space' AND ($1 IS NULL OR r.home_space_id=$1)
           AND r.completed_at IS NULL
         UNION ALL
         SELECT r.id,e.conversation_id,e.branch_id,c.title FROM spaces_runs r
         JOIN spaces_external_agent_conversations e ON e.run_id=r.id
         JOIN spaces_conversations c ON c.id=e.conversation_id
         WHERE r.scope_kind='space' AND ($1 IS NULL OR r.home_space_id=$1)
           AND r.completed_at IS NULL",
    )
    .bind(space_id.as_deref())
    .fetch_all(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    let mut runs = Vec::with_capacity(identities.len());
    for (run_id, conversation_id, branch_id, title) in identities {
        runs.push(ActiveConversationRun {
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
pub async fn get_conversation_task_error(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Option<String>, AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let model: Option<Option<String>> =
        sqlx::query_scalar("SELECT error FROM spaces_builtin_agent_tasks WHERE run_id=$1")
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
