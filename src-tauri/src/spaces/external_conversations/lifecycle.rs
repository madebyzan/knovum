use tauri::AppHandle;

use crate::{foundation::app_error::AppError, spaces::db};

/// Publishes what a finished turn left in its conversation's workspace and settles the turn.
pub async fn close(app: AppHandle, binding_id: &str) -> Result<(), AppError> {
    super::updates::release(binding_id);
    let pool = db::pool(&app).await?;
    let (run_id, space_id, conversation_id) = owner(pool, binding_id).await?;
    let workspace = workspace(&app, &space_id, &conversation_id)?;
    if let Err(error) =
        super::publication::publish(&app, binding_id, &run_id, &space_id, &workspace).await
    {
        record_publication_failure(pool, binding_id, &error.message).await?;
        return Err(error);
    }
    crate::spaces::conversations::extensions::release_run(&app, pool, &run_id).await?;
    Ok(())
}

/// The Run a turn records, and the Space conversation whose workspace it writes to.
async fn owner(
    pool: &sqlx::SqlitePool,
    binding_id: &str,
) -> Result<(String, String, String), String> {
    sqlx::query_as(
        "SELECT run_id,space_id,conversation_id FROM spaces_external_agent_conversations WHERE id=$1",
    )
    .bind(binding_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)
}

fn workspace(
    app: &AppHandle,
    space_id: &str,
    conversation_id: &str,
) -> Result<std::path::PathBuf, String> {
    crate::spaces::agent_artifacts::workspace::session_directory(app, space_id, conversation_id)
        .map_err(|error| error.to_string())
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

/// Cancels the turn recording `run_id`. The Run is settled as cancelled first, so the turn —
/// which stops when the Agent answers its cancellation — settles into it; what the turn left in
/// the workspace is dropped once it has stopped. The conversation's runtime stays for the next
/// turn.
pub async fn cancel_by_run(app: &AppHandle, run_id: &str) -> Result<bool, String> {
    let pool = db::pool(app).await?;
    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id,space_id,conversation_id FROM spaces_external_agent_conversations WHERE run_id=$1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some((binding_id, space_id, conversation_id)) = row else {
        return Ok(false);
    };
    let _ownership = super::publication_ownership::acquire(run_id).await?;
    crate::spaces::runs::drain_children(pool, run_id).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET state='closed',
         error='Cancelled by user',updated_at=$1 WHERE id=$2",
    )
    .bind(now)
    .bind(&binding_id)
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
    super::runtime::pool::stop(&binding_id).await;
    super::updates::release(&binding_id);
    super::staging::cleanup(app, pool, run_id).await?;
    crate::external_agents::artifacts::clear(&workspace(app, &space_id, &conversation_id)?)
        .map_err(|error| error.to_string())?;
    crate::spaces::conversations::extensions::release_run(app, pool, run_id).await?;
    Ok(true)
}

/// Deletes one turn's records. Its conversation's workspace is not the turn's to delete.
pub async fn delete(app: &AppHandle, binding_id: &str) -> Result<(), String> {
    if super::runtime::pool::is_running(binding_id) {
        return Err("external-conversation-active: close the session before deleting it".into());
    }
    let pool = db::pool(app).await?;
    let run_id: String =
        sqlx::query_scalar("SELECT run_id FROM spaces_external_agent_conversations WHERE id=$1")
            .bind(binding_id)
            .fetch_one(pool)
            .await
            .map_err(db::sql_error)?;
    let _ownership = super::publication_ownership::acquire(&run_id).await?;
    let settled: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM spaces_external_agent_conversations WHERE id=$1 AND output_publication_state<>'publishing'")
            .bind(binding_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?;
    settled.ok_or("external-conversation-unavailable")?;
    super::staging::cleanup(app, pool, &run_id).await?;
    crate::spaces::conversations::extensions::release_run(app, pool, &run_id).await?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query("DELETE FROM spaces_action_invocations WHERE principal_kind='external_agent' AND principal_id=$1")
        .bind(binding_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query(
        "WITH RECURSIVE run_tree(id) AS (
         SELECT id FROM spaces_runs WHERE principal_kind='external_agent' AND principal_id=$1
         UNION ALL SELECT child.id FROM spaces_runs child JOIN run_tree parent
           ON child.parent_run_id=parent.id
       ) DELETE FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)",
    )
    .bind(binding_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_authorization_grants WHERE principal_kind='external_agent' AND principal_id=$1")
        .bind(binding_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
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
    let finished: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT c.id,c.run_id,c.space_id,c.conversation_id FROM spaces_external_agent_conversations c
         JOIN spaces_runs r ON r.id=c.run_id
         WHERE c.output_publication_state<>'committed' AND r.cancel_requested_at IS NULL
         AND r.state<>'cancelled' AND EXISTS(SELECT 1 FROM spaces_external_agent_turns t
                    WHERE t.conversation_id=c.id AND t.state='completed')",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (id, run_id, space_id, conversation_id) in finished {
        let workspace = workspace(&app, &space_id, &conversation_id)?;
        if let Err(error) =
            super::publication::publish(&app, &id, &run_id, &space_id, &workspace).await
        {
            log::error!("Agent output recovery failed for {id}: {error}");
            record_publication_failure(pool, &id, &error.message).await?;
        }
    }
    let now = crate::foundation::clock::unix_time_millis()?;
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
        crate::spaces::conversations::extensions::release_run(&app, pool, &id).await?;
    }
    crate::spaces::runs::announce_change();
    Ok(())
}
