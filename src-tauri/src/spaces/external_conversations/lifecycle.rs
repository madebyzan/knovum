use tauri::AppHandle;

use crate::{ai::capabilities, app_error::AppError, spaces::db};

pub async fn close(app: AppHandle, conversation_id: &str) -> Result<(), AppError> {
    crate::external_agents::conversation::finish(conversation_id).await?;
    super::updates::release(conversation_id);
    let pool = db::pool(&app).await?;
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT run_id,space_id FROM spaces_external_agent_conversations WHERE id=$1",
    )
    .bind(conversation_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    let workspace = capabilities::agent_artifacts::workspace(&app, &row.1, conversation_id)
        .map_err(|error| error.to_string())?;
    if let Err(error) =
        super::publication::publish(&app, conversation_id, &row.0, &row.1, &workspace).await
    {
        record_publication_failure(pool, conversation_id, &error.message).await?;
        return Err(error);
    }
    crate::extensions::reclamation::release_run(&app, pool, &row.0).await?;
    Ok(())
}

async fn record_publication_failure(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
    error: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET output_publication_state='failed',
        output_publication_error=$1 WHERE id=$2 AND output_publication_state<>'committed'
        AND state<>'closed' AND EXISTS(SELECT 1 FROM spaces_runs r WHERE r.id=run_id AND r.cancel_requested_at IS NULL)",
    )
    .bind(error)
    .bind(conversation_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub async fn cancel_by_run(app: &AppHandle, run_id: &str) -> Result<bool, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT id,space_id FROM spaces_external_agent_conversations WHERE run_id=$1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some((conversation_id, space_id)) = row else {
        return Ok(false);
    };
    if crate::external_agents::conversation::is_active(&conversation_id) {
        crate::external_agents::conversation::close(&conversation_id).await?;
    }
    super::updates::release(&conversation_id);
    let _ownership = super::publication_ownership::acquire(run_id).await?;
    crate::spaces::runs::drain_children(pool, run_id).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET state='closed',
         error='Cancelled by user',updated_at=$1 WHERE id=$2",
    )
    .bind(now)
    .bind(&conversation_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='cancelled',state_version=state_version+1,
         updated_at=$1,completed_at=$1 WHERE id=$2 AND completed_at IS NULL",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    super::staging::cleanup(app, pool, run_id).await?;
    let workspace = capabilities::agent_artifacts::workspace(app, &space_id, &conversation_id)
        .map_err(|error| error.to_string())?;
    capabilities::agent_artifacts::discard(&workspace).map_err(|error| error.to_string())?;
    crate::extensions::reclamation::release_run(app, pool, run_id).await?;
    Ok(true)
}

pub async fn delete(app: &AppHandle, conversation_id: &str) -> Result<(), String> {
    if crate::external_agents::conversation::is_active(conversation_id) {
        return Err("external-conversation-active: close the session before deleting it".into());
    }
    let pool = db::pool(app).await?;
    let run_id: String =
        sqlx::query_scalar("SELECT run_id FROM spaces_external_agent_conversations WHERE id=$1")
            .bind(conversation_id)
            .fetch_one(pool)
            .await
            .map_err(db::sql_error)?;
    let _ownership = super::publication_ownership::acquire(&run_id).await?;
    let space_id: String =
        sqlx::query_scalar("SELECT space_id FROM spaces_external_agent_conversations WHERE id=$1 AND output_publication_state<>'publishing'")
            .bind(conversation_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?
            .ok_or("external-conversation-unavailable")?;
    super::staging::cleanup(app, pool, &run_id).await?;
    crate::extensions::reclamation::release_run(app, pool, &run_id).await?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query("DELETE FROM spaces_action_invocations WHERE principal_kind='external_agent' AND principal_id=$1")
        .bind(conversation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query(
        "WITH RECURSIVE run_tree(id) AS (
         SELECT id FROM spaces_runs WHERE principal_kind='external_agent' AND principal_id=$1
         UNION ALL SELECT child.id FROM spaces_runs child JOIN run_tree parent
           ON child.parent_run_id=parent.id
       ) DELETE FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)",
    )
    .bind(conversation_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_authorization_grants WHERE principal_kind='external_agent' AND principal_id=$1")
        .bind(conversation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    let workspace = capabilities::agent_artifacts::workspace(app, &space_id, conversation_id)
        .map_err(|error| error.to_string())?;
    capabilities::agent_artifacts::discard(&workspace).map_err(|error| error.to_string())
}

pub async fn recover(app: AppHandle) -> Result<(), String> {
    let pool = db::pool(&app).await?;
    sqlx::query("UPDATE spaces_external_agent_conversations SET output_publication_state='failed',
        output_publication_error='Agent output publication was interrupted' WHERE output_publication_state='publishing'")
        .execute(pool).await.map_err(db::sql_error)?;
    let staged_runs: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT run_id FROM spaces_external_output_staging")
            .fetch_all(pool)
            .await
            .map_err(db::sql_error)?;
    for run_id in staged_runs {
        let _ownership = super::publication_ownership::acquire(&run_id).await?;
        super::staging::cleanup(&app, pool, &run_id).await?;
    }
    let finished: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT c.id,c.run_id,c.space_id FROM spaces_external_agent_conversations c
         JOIN spaces_runs r ON r.id=c.run_id
         WHERE c.output_publication_state<>'committed' AND r.cancel_requested_at IS NULL
         AND r.state<>'cancelled' AND EXISTS(SELECT 1 FROM spaces_external_agent_turns t
                    WHERE t.conversation_id=c.id AND t.state='completed')",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (id, run_id, space_id) in finished {
        let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &id)
            .map_err(|error| error.to_string())?;
        if let Err(error) =
            super::publication::publish(&app, &id, &run_id, &space_id, &workspace).await
        {
            log::error!("Agent output recovery failed for {id}: {error}");
            record_publication_failure(pool, &id, &error.message).await?;
        }
    }
    let committed: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id,space_id,run_id FROM spaces_external_agent_conversations WHERE state='closed'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (id, space_id, run_id) in committed {
        let _ownership = super::publication_ownership::acquire(&run_id).await?;
        let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &id)
            .map_err(|error| error.to_string())?;
        if let Err(error) = capabilities::agent_artifacts::discard(&workspace) {
            log::warn!("Could not remove committed Agent workspace {id}: {error}");
        }
    }
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET state='disconnected',
         error='External Agent session ended with the application',updated_at=$1
         WHERE state IN ('connecting','connected')",
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='failed',state_version=state_version+1,
         updated_at=$1,completed_at=$1 WHERE completed_at IS NULL AND id IN
         (SELECT run_id FROM spaces_external_agent_conversations WHERE state='disconnected')",
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    let finished_runs: Vec<String> = sqlx::query_scalar("SELECT c.run_id FROM spaces_external_agent_conversations c JOIN spaces_runs r ON r.id=c.run_id WHERE r.completed_at IS NOT NULL")
        .fetch_all(pool).await.map_err(db::sql_error)?;
    for id in finished_runs {
        crate::extensions::reclamation::release_run(&app, pool, &id).await?;
    }
    crate::spaces::runs::announce_change();
    Ok(())
}
