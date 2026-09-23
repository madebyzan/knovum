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
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query(
        "SELECT s.id,s.run_id,s.node_kind,s.configuration,s.resolved_input,s.output,
         r.home_space_id,r.policy_snapshot,r.budget,r.execution_epoch,
         COALESCE((SELECT MAX(attempt_number) FROM spaces_run_attempts WHERE step_instance_id=s.id),0)+1 attempt_number
         FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id
         WHERE s.state='ready' AND r.state IN ('queued','running')
           AND s.available_at <= $1
           AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL
           AND (SELECT COUNT(*) FROM spaces_run_steps active
                WHERE active.run_id=r.id AND active.state='running')
               < COALESCE(CAST(json_extract(r.budget,'$.maxParallelism') AS INTEGER),1)
         ORDER BY CASE r.kind WHEN 'assistant' THEN 0 WHEN 'action' THEN 1
                  WHEN 'workflow' THEN 2 WHEN 'automation' THEN 3 ELSE 4 END,
                  r.created_at,s.created_at LIMIT 1",
    )
    .bind(now).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
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
    Ok(Some(claim))
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
        let limit = budget
            .get("maxStepAttempts")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(i64::MAX);
        if attempts >= limit {
            exhaust_run(tx, run_id, now, "maxStepAttempts").await?;
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
    sqlx::query(
        "UPDATE spaces_run_steps SET state='blocked',wait_reason='budget',
         state_version=state_version+1,updated_at=$1 WHERE run_id=$2
         AND state IN ('pending','ready','waiting','paused')",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='failed',wait_reason='budget',state_version=state_version+1,
         execution_epoch=execution_epoch+1,updated_at=$1,completed_at=$1 WHERE id=$2 AND completed_at IS NULL",
    ).bind(now).bind(run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
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
