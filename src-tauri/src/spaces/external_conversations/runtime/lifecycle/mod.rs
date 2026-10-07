pub(super) mod create;
pub(super) mod termination;

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
    let connected = sqlx::query(
        "UPDATE spaces_external_agent_conversations SET state='connected',runtime_session_id=$1,
         session_capabilities=$2,updated_at=$3,error=NULL WHERE id=$4 AND run_id=$5 AND state='connecting'
         AND EXISTS(SELECT 1 FROM spaces_runs WHERE id=$5 AND completed_at IS NULL AND cancel_requested_at IS NULL)",
    )
    .bind(&session.runtime_session_id)
    .bind(capabilities)
    .bind(now)
    .bind(id)
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(connected, "external-conversation-startup-authority-revoked")?;
    sqlx::query("UPDATE spaces_runs SET state='running',state_version=state_version+1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL")
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
) -> Result<bool, String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let disconnected = sqlx::query("UPDATE spaces_external_agent_conversations SET state='disconnected',error=$1,updated_at=$2,
        output_publication_state=CASE WHEN output_publication_state='pending' AND EXISTS(
            SELECT 1 FROM spaces_external_agent_turns WHERE conversation_id=$3 AND state='completed')
            THEN 'failed' ELSE output_publication_state END,
        output_publication_error=CASE WHEN output_publication_state='pending' AND EXISTS(
            SELECT 1 FROM spaces_external_agent_turns WHERE conversation_id=$3 AND state='completed')
            THEN $1 ELSE output_publication_error END
        WHERE id=$3 AND run_id=$4 AND output_publication_state<>'committed' AND state IN ('connecting','connected')
        AND EXISTS(SELECT 1 FROM spaces_runs WHERE id=$4 AND cancel_requested_at IS NULL)")
        .bind(error).bind(now).bind(id).bind(run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if disconnected.rows_affected() == 0 {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(false);
    }
    sqlx::query(
        "UPDATE spaces_external_agent_turns SET state='failed',error=$1,completed_at=$2
        WHERE conversation_id=$3 AND state='running'",
    )
    .bind(error)
    .bind(now)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
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
    super::super::updates::release(id);
    crate::spaces::runs::announce_change();
    Ok(true)
}

pub(super) async fn load(
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<super::super::ExternalAgentSessionBinding, String> {
    let row: (String, String, String, String) = sqlx::query_as(
        "SELECT id,run_id,conversation_id,branch_id
         FROM spaces_external_agent_conversations WHERE id=$1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(super::super::ExternalAgentSessionBinding {
        id: row.0,
        run_id: row.1,
        conversation_id: row.2,
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

#[cfg(test)]
mod tests;
