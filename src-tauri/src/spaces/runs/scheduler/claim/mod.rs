use sqlx::{Row, SqlitePool};

use crate::spaces::{db, runs::AttemptClaim};

mod parsing;

const LEASE_MILLIS: i64 = 30_000;

pub(crate) async fn renew_lease(pool: &SqlitePool, claim: &AttemptClaim) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let changed = sqlx::query(
        "UPDATE spaces_run_attempts SET lease_until=$1 WHERE id=$2 AND state='executing'
         AND execution_epoch=$3",
    )
    .bind(now + LEASE_MILLIS)
    .bind(&claim.attempt_id)
    .bind(claim.execution_epoch)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "attempt-stale: execution lease is no longer active",
    )
}

pub(crate) async fn claim_ready_step(
    pool: &SqlitePool,
    executor_id: &str,
) -> Result<Option<AttemptClaim>, String> {
    let now = crate::clock::unix_time_millis()?;
    let ready: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id
         WHERE s.state='ready' AND s.available_at<=$1 AND r.state IN ('queued','running')
           AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL AND r.completed_at IS NULL)",
    ).bind(now).fetch_one(pool).await.map_err(db::sql_error)?;
    if !ready {
        return Ok(None);
    }
    let mut tx = db::begin_write(pool).await?;
    let rows = sqlx::query(
        "SELECT s.id,s.run_id,s.node_kind,s.configuration,s.resolved_input,
         r.scope_kind,r.home_space_id,r.policy_snapshot,r.budget,r.execution_epoch,
         COALESCE((SELECT MAX(attempt_number) FROM spaces_run_attempts WHERE step_instance_id=s.id),0)+1 attempt_number
         FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id
         WHERE s.state='ready' AND r.state IN ('queued','running')
           AND s.available_at <= $1
           AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL
           AND r.completed_at IS NULL
         ORDER BY CASE r.kind WHEN 'builtin_agent' THEN 0 WHEN 'action' THEN 1
                  WHEN 'workflow' THEN 2 ELSE 4 END,
                  r.created_at,s.created_at",
    )
    .bind(now).fetch_all(&mut *tx).await.map_err(db::sql_error)?;
    let mut row = None;
    for candidate in rows {
        let candidate_run: String = candidate.try_get("run_id").map_err(db::sql_error)?;
        if candidate_has_dispatch_authority(&mut tx, &candidate_run).await? {
            row = Some(candidate);
            break;
        }
    }
    let Some(row) = row else {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(None);
    };
    let step_id: String = row.try_get("id").map_err(db::sql_error)?;
    let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
    let epoch: i64 = row.try_get("execution_epoch").map_err(db::sql_error)?;
    let attempt_number: i64 = row.try_get("attempt_number").map_err(db::sql_error)?;
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let lease_until = now + LEASE_MILLIS;
    if !reserve_attempt_budget(&mut tx, &run_id, now).await? {
        tx.commit().await.map_err(db::sql_error)?;
        super::super::signals::announce();
        return Ok(None);
    }
    let claimed = sqlx::query("UPDATE spaces_run_steps SET state='running',state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='ready'")
        .bind(now).bind(&step_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if claimed.rows_affected() != 1 {
        tx.rollback().await.map_err(db::sql_error)?;
        return Ok(None);
    }
    sqlx::query("UPDATE spaces_runs SET state='running',state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='queued'")
        .bind(now).bind(&run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    let resolved_input: String = row.try_get("resolved_input").map_err(db::sql_error)?;
    let digest = parsing::request_digest(&resolved_input);
    sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at) VALUES($1,$2,$3,$4,'executing',$5,$6,$7,$8,$9,$10)")
        .bind(&attempt_id).bind(&run_id).bind(&step_id).bind(attempt_number).bind(digest)
        .bind(executor_id).bind(&step_id).bind(epoch).bind(lease_until).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    super::super::append_event(
        &mut tx,
        &run_id,
        Some(&step_id),
        Some(&attempt_id),
        crate::spaces::runs::RunEventKind::AttemptClaimed,
        &serde_json::json!({"executorId": executor_id, "executionEpoch": epoch}),
        now,
    )
    .await?;
    let claim = parsing::claim(&row, attempt_id, step_id, run_id)?;
    tx.commit().await.map_err(db::sql_error)?;
    super::super::signals::announce();
    Ok(Some(claim))
}

async fn candidate_has_dispatch_authority(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    run_id: &str,
) -> Result<bool, String> {
    let ancestors = sqlx::query(
        "WITH RECURSIVE ancestors(id,parent_run_id,budget,cancel_requested_at,pause_requested_at) AS (
           SELECT id,parent_run_id,budget,cancel_requested_at,pause_requested_at
           FROM spaces_runs WHERE id=$1
           UNION ALL SELECT parent.id,parent.parent_run_id,parent.budget,parent.cancel_requested_at,
           parent.pause_requested_at FROM spaces_runs parent
           JOIN ancestors child ON child.parent_run_id=parent.id
         ) SELECT id,budget,cancel_requested_at,pause_requested_at FROM ancestors",
    )
    .bind(run_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for ancestor in ancestors {
        let cancelled: Option<i64> = ancestor
            .try_get("cancel_requested_at")
            .map_err(db::sql_error)?;
        let paused: Option<i64> = ancestor
            .try_get("pause_requested_at")
            .map_err(db::sql_error)?;
        if cancelled.is_some() || paused.is_some() {
            return Ok(false);
        }
        let ancestor_id: String = ancestor.try_get("id").map_err(db::sql_error)?;
        let budget: String = ancestor.try_get("budget").map_err(db::sql_error)?;
        let budget: serde_json::Value = serde_json::from_str(&budget).map_err(db::json_error)?;
        let limit = super::super::budget::optional_limit(&budget, "maxParallelism")?.unwrap_or(1);
        let running: i64 = sqlx::query_scalar(
            "WITH RECURSIVE run_tree(id) AS (
               SELECT id FROM spaces_runs WHERE id=$1
               UNION ALL SELECT child.id FROM spaces_runs child
               JOIN run_tree parent ON child.parent_run_id=parent.id
             ) SELECT COUNT(*) FROM spaces_run_steps
             WHERE state='running' AND run_id IN (SELECT id FROM run_tree)",
        )
        .bind(ancestor_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if running >= limit {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn reserve_attempt_budget(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    run_id: &str,
    now: i64,
) -> Result<bool, String> {
    let rows = sqlx::query(
        "WITH RECURSIVE ancestors(id,parent_run_id,step_attempts,budget) AS (
           SELECT id,parent_run_id,step_attempts,budget FROM spaces_runs WHERE id=$1
           UNION ALL
           SELECT parent.id,parent.parent_run_id,parent.step_attempts,parent.budget
           FROM spaces_runs parent JOIN ancestors child ON child.parent_run_id=parent.id
         ) SELECT id,step_attempts,budget FROM ancestors",
    )
    .bind(run_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let attempts: i64 = row.try_get("step_attempts").map_err(db::sql_error)?;
        let budget: String = row.try_get("budget").map_err(db::sql_error)?;
        let budget: serde_json::Value = serde_json::from_str(&budget).map_err(db::json_error)?;
        let limit =
            super::super::budget::optional_limit(&budget, "maxStepAttempts")?.unwrap_or(i64::MAX);
        if attempts >= limit {
            exhaust_run(tx, &id, now, "maxStepAttempts").await?;
            return Ok(false);
        }
        ids.push(id);
    }
    for id in ids {
        sqlx::query(
            "UPDATE spaces_runs SET step_attempts=step_attempts+1,updated_at=$1 WHERE id=$2",
        )
        .bind(now)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(true)
}

async fn exhaust_run(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    run_id: &str,
    now: i64,
    budget: &str,
) -> Result<(), String> {
    super::controls::exhaust_run_tree(tx, run_id, now, "run-attempt-budget-exceeded").await?;
    super::super::append_event(
        tx,
        run_id,
        None,
        None,
        crate::spaces::runs::RunEventKind::RunBudgetExhausted,
        &serde_json::json!({"budget": budget}),
        now,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests;
