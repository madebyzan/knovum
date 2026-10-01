use sqlx::SqlitePool;

use crate::spaces::db;

pub(super) async fn recover_expired_leases(pool: &SqlitePool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let expired_epochs: Vec<(String, i64)> = sqlx::query_as(
        "SELECT DISTINCT a.run_id,a.execution_epoch FROM spaces_run_attempts a
         JOIN spaces_run_steps s ON s.id=a.step_instance_id AND s.run_id=a.run_id
         JOIN spaces_runs r ON r.id=a.run_id AND r.execution_epoch=a.execution_epoch
         WHERE a.state IN ('claimed','executing') AND a.lease_until < $1 AND s.state='running'
         AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL",
    )
    .bind(now)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let changed = !expired_epochs.is_empty();
    for (run_id, epoch) in expired_epochs {
        let step_ids: Vec<String> = sqlx::query_scalar(
            "UPDATE spaces_run_attempts SET state='outcome_unknown',finished_at=$1,
             error_code='lease_expired' WHERE run_id=$2 AND execution_epoch=$3
             AND state IN ('claimed','executing') RETURNING step_instance_id",
        )
        .bind(now)
        .bind(&run_id)
        .bind(epoch)
        .fetch_all(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        let mut blocked = false;
        for step_id in step_ids {
            let step = sqlx::query(
                "UPDATE spaces_run_steps SET state='blocked',wait_reason='external_reconciliation',
                 state_version=state_version+1,updated_at=$1
                 WHERE id=$2 AND run_id=$3 AND state='running'",
            )
            .bind(now)
            .bind(step_id)
            .bind(&run_id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
            blocked |= step.rows_affected() == 1;
        }
        if !blocked {
            return Err("run-stale: expired epoch has no running steps".into());
        }
        let run = sqlx::query(
            "UPDATE spaces_runs SET state='waiting',wait_reason='external_reconciliation',
             state_version=state_version+1,execution_epoch=execution_epoch+1,updated_at=$1
             WHERE id=$2 AND execution_epoch=$3 AND cancel_requested_at IS NULL
             AND completed_at IS NULL",
        )
        .bind(now)
        .bind(run_id)
        .bind(epoch)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        db::require_one(
            run,
            "run-stale: expired attempt owner changed during recovery",
        )?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    if changed {
        super::super::signals::announce();
    }
    Ok(())
}

pub(super) async fn apply(pool: &SqlitePool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    propagate_cancellation(&mut tx, now).await?;
    let expired = expire_duration_budgets(&mut tx, now).await?;
    sqlx::query("UPDATE spaces_run_attempts SET state='cancelled',lease_until=NULL,finished_at=$1,error_code='run_cancelled' WHERE state IN ('claimed','executing','waiting_external') AND run_id IN (SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL)")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_run_steps SET state='cancelled',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE state IN ('pending','ready','running','waiting','paused','blocked') AND run_id IN (SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL)")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    let cancelled = sqlx::query("UPDATE spaces_runs SET state='cancelled',state_version=state_version+1,updated_at=$1,completed_at=$1 WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?.rows_affected();
    let paused = sqlx::query("UPDATE spaces_runs SET state='paused',state_version=state_version+1,wait_reason=NULL,updated_at=$1 WHERE pause_requested_at IS NOT NULL AND state IN ('queued','running','waiting') AND NOT EXISTS (SELECT 1 FROM spaces_run_steps WHERE run_id=spaces_runs.id AND state='running')")
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
       SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL
       UNION SELECT child.id FROM spaces_runs child JOIN cancelled_tree parent ON child.parent_run_id=parent.id
       ) UPDATE spaces_runs SET cancel_requested_at=$1,
       execution_epoch=execution_epoch+1,updated_at=$1 WHERE id IN (SELECT id FROM cancelled_tree)
       AND cancel_requested_at IS NULL AND completed_at IS NULL")
        .bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

async fn expire_duration_budgets(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    now: i64,
) -> Result<u64, String> {
    let roots: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_runs WHERE completed_at IS NULL AND cancel_requested_at IS NULL
         AND created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER) * 1000 <= $1",
    )
    .bind(now)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut changed = 0;
    for root_id in roots {
        changed += exhaust_run_tree(tx, &root_id, now, "run_duration_exceeded").await?;
    }
    Ok(changed)
}

pub(super) async fn exhaust_run_tree(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    root_id: &str,
    now: i64,
    error_code: &str,
) -> Result<u64, String> {
    const TREE: &str = "WITH RECURSIVE run_tree(id) AS (
       SELECT id FROM spaces_runs WHERE id=$2
       UNION ALL SELECT child.id FROM spaces_runs child JOIN run_tree parent
       ON child.parent_run_id=parent.id)";
    sqlx::query(&format!(
        "{TREE} UPDATE spaces_run_attempts SET state='cancelled',lease_until=NULL,finished_at=$1,
         error_code=$3 WHERE state IN ('claimed','executing','waiting_external')
         AND run_id IN (SELECT id FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)
         AND completed_at IS NULL AND cancel_requested_at IS NULL)"
    ))
    .bind(now)
    .bind(root_id)
    .bind(error_code)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(&format!(
        "{TREE} UPDATE spaces_run_steps SET state='blocked',wait_reason='budget',
         state_version=state_version+1,updated_at=$1
         WHERE state IN ('pending','ready','running','waiting','paused','blocked')
         AND run_id IN (SELECT id FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)
         AND completed_at IS NULL AND cancel_requested_at IS NULL)"
    ))
    .bind(now)
    .bind(root_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let runs = sqlx::query(&format!(
        "{TREE} UPDATE spaces_runs SET state='failed',wait_reason='budget',
         state_version=state_version+1,execution_epoch=execution_epoch+1,updated_at=$1,
         completed_at=$1 WHERE id IN (SELECT id FROM run_tree)
         AND completed_at IS NULL AND cancel_requested_at IS NULL"
    ))
    .bind(now)
    .bind(root_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(runs.rows_affected())
}

#[cfg(test)]
mod tests;
