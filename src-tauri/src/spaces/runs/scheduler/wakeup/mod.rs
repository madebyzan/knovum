use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::watch;

use crate::spaces::db;

pub(in crate::spaces::runs) async fn next_deadline(
    pool: &SqlitePool,
    now: i64,
) -> Result<Option<i64>, String> {
    sqlx::query_scalar(
        "SELECT MIN(deadline) FROM (
           SELECT MIN(created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER) * 1000) AS deadline
           FROM spaces_runs WHERE completed_at IS NULL AND cancel_requested_at IS NULL
           UNION ALL
           SELECT MIN(a.lease_until + 1) FROM spaces_run_attempts a
           JOIN spaces_runs r ON r.id=a.run_id AND r.execution_epoch=a.execution_epoch
           JOIN spaces_run_steps s ON s.id=a.step_instance_id AND s.run_id=a.run_id
           WHERE a.state IN ('claimed','executing') AND s.state='running'
             AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL
           UNION ALL
           SELECT MIN(s.available_at) FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id
           WHERE s.state='ready' AND s.available_at>$1 AND r.state IN ('queued','running')
             AND r.completed_at IS NULL AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL
         )",
    )
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)
}

pub(in crate::spaces::runs) async fn wait(
    changes: &mut watch::Receiver<u64>,
    deadline: Option<i64>,
) -> Result<(), String> {
    match deadline {
        Some(deadline) => {
            let now = crate::foundation::clock::unix_time_millis()?;
            let remaining = Duration::from_millis(deadline.saturating_sub(now).max(0) as u64);
            tokio::select! {
                changed = changes.changed() => changed.map_err(|_| "Run change signal closed".to_owned()),
                _ = tokio::time::sleep(remaining) => Ok(()),
            }
        }
        None => changes
            .changed()
            .await
            .map_err(|_| "Run change signal closed".to_owned()),
    }
}

#[cfg(test)]
mod tests;
