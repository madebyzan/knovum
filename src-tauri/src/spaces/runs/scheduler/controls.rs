use sqlx::SqlitePool;

use crate::spaces::db;

pub(super) async fn recover_expired_leases(pool: &SqlitePool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query("UPDATE spaces_run_attempts SET state='outcome_unknown',finished_at=$1,error_code='lease-expired' WHERE state IN ('claimed','executing') AND lease_until < $1")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_run_steps SET state='blocked',wait_reason='external_reconciliation',state_version=state_version+1,updated_at=$1 WHERE state='running' AND id IN (SELECT step_instance_id FROM spaces_run_attempts WHERE state='outcome_unknown')")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_runs SET state='waiting',wait_reason='external_reconciliation',state_version=state_version+1,execution_epoch=execution_epoch+1,updated_at=$1 WHERE completed_at IS NULL AND id IN (SELECT run_id FROM spaces_run_steps WHERE state='blocked' AND wait_reason='external_reconciliation')")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn apply(pool: &SqlitePool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    propagate_cancellation(&mut tx, now).await?;
    let expired = expire_duration_budgets(&mut tx, now).await?;
    sqlx::query("UPDATE spaces_run_attempts SET state='cancelled',lease_until=NULL,finished_at=$1,error_code='run-cancelled' WHERE state IN ('claimed','executing','waiting_external') AND run_id IN (SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL)")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_run_steps SET state='cancelled',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE state IN ('pending','ready','running','waiting','paused','blocked') AND run_id IN (SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL)")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    let cancelled = sqlx::query("UPDATE spaces_runs SET state='cancelled',state_version=state_version+1,updated_at=$1,completed_at=$1 WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?.rows_affected();
    let paused = sqlx::query("UPDATE spaces_runs SET state='paused',state_version=state_version+1,wait_reason=NULL,updated_at=$1 WHERE pause_requested_at IS NOT NULL AND state IN ('queued','running') AND NOT EXISTS (SELECT 1 FROM spaces_run_steps WHERE run_id=spaces_runs.id AND state='running')")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?.rows_affected();
    tx.commit().await.map_err(db::sql_error)?;
    if cancelled + paused + expired > 0 {
        super::super::signals::announce();
    }
    Ok(())
}

async fn propagate_cancellation(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    now: i64,
) -> Result<(), String> {
    sqlx::query("WITH RECURSIVE cancelled_tree(id) AS (
       SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL
       UNION SELECT child.id FROM spaces_runs child JOIN cancelled_tree parent ON child.parent_run_id=parent.id
       WHERE child.completed_at IS NULL) UPDATE spaces_runs SET cancel_requested_at=COALESCE(cancel_requested_at,$1),
       execution_epoch=execution_epoch+CASE WHEN cancel_requested_at IS NULL THEN 1 ELSE 0 END,
       updated_at=$1 WHERE id IN (SELECT id FROM cancelled_tree)")
        .bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

async fn expire_duration_budgets(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    now: i64,
) -> Result<u64, String> {
    let expired = "SELECT id FROM spaces_runs WHERE completed_at IS NULL AND created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER) * 1000 <= $1";
    sqlx::query(&format!("UPDATE spaces_run_attempts SET state='cancelled',lease_until=NULL,finished_at=$1,error_code='run-duration-exceeded' WHERE state IN ('claimed','executing','waiting_external') AND run_id IN ({expired})"))
        .bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query(&format!("UPDATE spaces_run_steps SET state='blocked',wait_reason='budget',state_version=state_version+1,updated_at=$1 WHERE state IN ('pending','ready','running','waiting','paused') AND run_id IN ({expired})"))
        .bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    let result = sqlx::query("UPDATE spaces_runs SET state='failed',wait_reason='budget',state_version=state_version+1,execution_epoch=execution_epoch+1,updated_at=$1,completed_at=$1 WHERE completed_at IS NULL AND created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER) * 1000 <= $1")
        .bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(result.rows_affected())
}
