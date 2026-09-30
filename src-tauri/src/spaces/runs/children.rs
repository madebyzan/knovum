use super::RunState;
use crate::spaces::db;
use sqlx::{Sqlite, SqlitePool, Transaction};

/// Session owners stop accepting Actions before draining their durable descendants.
pub(crate) async fn wait_for_children(pool: &SqlitePool, run_id: &str) -> Result<RunState, String> {
    require_publishable_owner(pool, run_id).await?;
    let state = drain_children(pool, run_id).await?;
    require_publishable_owner(pool, run_id).await?;
    Ok(state)
}

async fn require_publishable_owner(pool: &SqlitePool, run_id: &str) -> Result<(), String> {
    let active: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_runs WHERE id=$1 AND (completed_at IS NULL OR state='failed')
         AND cancel_requested_at IS NULL AND pause_requested_at IS NULL",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    active
        .map(|_| ())
        .ok_or_else(|| "parent-run-inactive: Run was cancelled, paused, or completed".into())
}

pub(crate) async fn drain_children(pool: &SqlitePool, run_id: &str) -> Result<RunState, String> {
    let mut changes = super::subscribe_to_changes();
    loop {
        let mut tx = pool.begin().await.map_err(db::sql_error)?;
        let settlement = child_settlement(&mut tx, run_id).await?;
        tx.commit().await.map_err(db::sql_error)?;
        if let Some(state) = settlement {
            return Ok(state);
        }
        changes.changed().await.map_err(|error| error.to_string())?;
    }
}

pub(crate) async fn child_settlement(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
) -> Result<Option<RunState>, String> {
    let (pending, failed): (i64, i64) = sqlx::query_as(
        "WITH RECURSIVE descendants(id) AS (
         SELECT id FROM spaces_runs WHERE parent_run_id=$1
         UNION ALL SELECT child.id FROM spaces_runs child JOIN descendants parent ON child.parent_run_id=parent.id)
         SELECT COUNT(*) FILTER (WHERE completed_at IS NULL),
         COUNT(*) FILTER (WHERE state IN ('failed','cancelled','completed_with_errors'))
         FROM spaces_runs WHERE id IN (SELECT id FROM descendants)",
    ).bind(run_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    Ok((pending == 0).then_some(if failed == 0 {
        RunState::Succeeded
    } else {
        RunState::CompletedWithErrors
    }))
}
