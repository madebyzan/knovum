mod create;
pub(super) use create::create_records;

use tauri::AppHandle;

use super::StartExternalConversationRequest;
use crate::spaces::db;

pub(super) async fn mark_connected(
    pool: &sqlx::SqlitePool,
    id: &str,
    run_id: &str,
    session: &crate::external_agents::conversation::ConversationSession,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let capabilities = serde_json::to_string(&session.capabilities).map_err(db::json_error)?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET state='connected',runtime_session_id=$1,
         session_capabilities=$2,updated_at=$3,error=NULL WHERE id=$4",
    )
    .bind(&session.runtime_session_id)
    .bind(capabilities)
    .bind(now)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_runs SET state='running',state_version=1,updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

pub(crate) async fn mark_disconnected(
    pool: &sqlx::SqlitePool,
    id: &str,
    run_id: &str,
    error: &str,
) -> Result<(), String> {
    super::super::updates::release(id);
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query("UPDATE spaces_external_agent_conversations SET state='disconnected',error=$1,updated_at=$2 WHERE id=$3")
        .bind(error).bind(now).bind(id).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='failed',state_version=state_version+1,updated_at=$1,
         completed_at=$1 WHERE id=$2 AND completed_at IS NULL",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

pub(super) async fn load(
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<super::super::AgentSessionBinding, String> {
    let row: (String, String, String, String) = sqlx::query_as(
        "SELECT id,run_id,assistant_conversation_id,branch_id
         FROM spaces_external_agent_conversations WHERE id=$1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(super::super::AgentSessionBinding {
        id: row.0,
        run_id: row.1,
        assistant_conversation_id: row.2,
        branch_id: row.3,
    })
}

pub(super) fn validate(request: &StartExternalConversationRequest) -> Result<(), String> {
    if request.space_id.trim().is_empty()
        || request.title.trim().is_empty()
        || request.title.chars().count() > 160
        || !(30..=7_200).contains(&request.max_duration_seconds)
    {
        return Err("external-conversation-invalid".into());
    }
    Ok(())
}

pub(super) fn schedule_expiration(
    app: AppHandle,
    conversation_id: String,
    run_id: String,
    seconds: u32,
) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(u64::from(seconds))).await;
        if crate::external_agents::conversation::is_active(&conversation_id) {
            let _ = crate::external_agents::conversation::close(&conversation_id).await;
            if let Ok(pool) = db::pool(&app).await {
                let _ = mark_disconnected(
                    pool,
                    &conversation_id,
                    &run_id,
                    "External Agent conversation duration budget was exhausted",
                )
                .await;
            }
        }
    });
}
