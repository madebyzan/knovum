use tauri::AppHandle;

use crate::{ai::capabilities, spaces::db};

pub async fn close(app: AppHandle, conversation_id: &str) -> Result<(), String> {
    crate::external_agents::conversation::close(conversation_id).await?;
    super::updates::release(conversation_id);
    let pool = db::pool(&app).await?;
    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT run_id,space_id,runtime_kind FROM spaces_external_agent_conversations WHERE id=$1",
    )
    .bind(conversation_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    let workspace = capabilities::agent_artifacts::workspace(&app, &row.1, conversation_id)
        .map_err(|error| error.to_string())?;
    let has_artifacts = !capabilities::agent_artifacts::collect(&workspace)
        .map_err(|error| error.to_string())?
        .is_empty();
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET state='closed',updated_at=$1 WHERE id=$2",
    )
    .bind(now)
    .bind(conversation_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='succeeded',state_version=state_version+1,
         updated_at=$1,completed_at=$1 WHERE id=$2 AND completed_at IS NULL",
    )
    .bind(now)
    .bind(&row.0)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if has_artifacts {
        sqlx::query(
            "INSERT INTO spaces_agent_artifact_sessions(
             id,run_id,space_id,runtime_kind,state,created_at,updated_at,completed_at)
             VALUES($1,$2,$3,$4,'completed',$5,$5,$5)",
        )
        .bind(conversation_id)
        .bind(&row.0)
        .bind(&row.1)
        .bind(&row.2)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    } else {
        capabilities::agent_artifacts::discard(&workspace).map_err(|error| error.to_string())?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
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
    let workspace = capabilities::agent_artifacts::workspace(app, &space_id, &conversation_id)
        .map_err(|error| error.to_string())?;
    capabilities::agent_artifacts::discard(&workspace).map_err(|error| error.to_string())?;
    Ok(true)
}

pub async fn delete(app: &AppHandle, conversation_id: &str) -> Result<(), String> {
    if crate::external_agents::conversation::is_active(conversation_id) {
        return Err("external-conversation-active: close the session before deleting it".into());
    }
    let pool = db::pool(app).await?;
    let space_id: String =
        sqlx::query_scalar("SELECT space_id FROM spaces_external_agent_conversations WHERE id=$1")
            .bind(conversation_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?
            .ok_or("external-conversation-unavailable")?;
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
    crate::spaces::runs::announce_change();
    Ok(())
}
