use tauri::AppHandle;

use crate::spaces::db;

/// Sessions are no longer started; rows a crash left `running` (and their Runs) are settled as
/// failed once, so the ledger never reports work that will not finish.
pub(super) async fn fail_interrupted(app: &AppHandle) -> Result<Vec<(String, String)>, String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id,space_id FROM spaces_agent_artifact_sessions WHERE state='running'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_agent_artifact_sessions SET state='failed',
         error='Agent artifact execution was interrupted',updated_at=$1,completed_at=$1
         WHERE state='running'",
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='failed',state_version=state_version+1,updated_at=$1,
         completed_at=$1 WHERE completed_at IS NULL AND id IN
         (SELECT run_id FROM spaces_agent_artifact_sessions WHERE state='failed')",
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(rows)
}
