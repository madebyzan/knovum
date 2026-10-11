use sqlx::SqlitePool;

use crate::spaces::db;

const CANCELLATION_TREE: &str = "WITH RECURSIVE cancelled_tree(id) AS (
    SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL
    UNION SELECT child.id FROM spaces_runs child JOIN cancelled_tree parent ON child.parent_run_id=parent.id
)";

pub(super) async fn recover_expired_leases(pool: &SqlitePool) -> Result<bool, String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let expired: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_run_attempts a
         JOIN spaces_run_steps s ON s.id=a.step_instance_id AND s.run_id=a.run_id
         JOIN spaces_runs r ON r.id=a.run_id AND r.execution_epoch=a.execution_epoch
         WHERE a.state IN ('claimed','executing') AND a.lease_until<$1 AND s.state='running'
         AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL)",
    )
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if !expired {
        return Ok(false);
    }
    let mut tx = db::begin_write(pool).await?;
    let local_replays = recover_local_speech_attempts(&mut tx, now).await?;
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
    let changed = local_replays > 0 || !expired_epochs.is_empty();
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
    Ok(changed)
}

async fn recover_local_speech_attempts(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    now: i64,
) -> Result<u64, String> {
    let epochs: Vec<(String, i64)> = sqlx::query_as(
        "SELECT DISTINCT r.id,r.execution_epoch \
         FROM spaces_runs r \
         JOIN spaces_run_attempts expired ON expired.run_id=r.id \
           AND expired.execution_epoch=r.execution_epoch \
         JOIN spaces_run_steps expired_step ON expired_step.id=expired.step_instance_id \
           AND expired_step.run_id=r.id \
         WHERE expired.state IN ('claimed','executing') AND expired.lease_until < ? \
           AND expired_step.state='running' AND r.cancel_requested_at IS NULL \
           AND r.completed_at IS NULL \
           AND json_extract(r.input_snapshot,'$.workflow') IN ('speech_project_generation_v1','speech_source_separation_v1','speech_voice_candidate_v1','speech_video_translation_v1') \
           AND expired_step.node_kind IN ('speech_segment','speech_chapter_pcm','speech_loudness_measurement','speech_audiobook_encode','speech_assembly','speech_source_separation','speech_voice_candidate','speech_video_translation_segment') \
           AND NOT EXISTS(SELECT 1 FROM spaces_run_steps mixed WHERE mixed.run_id=r.id \
             AND mixed.node_kind NOT IN ('speech_segment','speech_chapter_pcm','speech_loudness_measurement','speech_audiobook_encode','speech_assembly','speech_source_separation','speech_voice_candidate','speech_video_translation_segment'))",
    )
    .bind(now).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let mut recovered = 0_u64;
    for (run_id, epoch) in epochs {
        let attempts: Vec<(String, String, bool)> = sqlx::query_as(
            "SELECT a.id,a.step_instance_id, \
               a.attempt_number < COALESCE(json_extract(r.budget,'$.maxAttempts'),3) AS retry \
             FROM spaces_run_attempts a \
             JOIN spaces_run_steps s ON s.id=a.step_instance_id AND s.run_id=a.run_id \
             JOIN spaces_runs r ON r.id=a.run_id AND r.execution_epoch=a.execution_epoch \
             WHERE a.run_id=? AND a.execution_epoch=? AND a.state IN ('claimed','executing') \
               AND s.state='running' AND s.node_kind IN ('speech_segment','speech_chapter_pcm','speech_loudness_measurement','speech_audiobook_encode','speech_assembly','speech_source_separation','speech_voice_candidate','speech_video_translation_segment')",
        ).bind(&run_id).bind(epoch).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
        if attempts.is_empty() {
            return Err("run-stale: local speech epoch has no active attempts".into());
        }
        let retry = attempts.iter().all(|attempt| attempt.2);
        for (attempt_id, step_id, _) in &attempts {
            let attempt = sqlx::query(
                "UPDATE spaces_run_attempts SET state='failed',lease_until=NULL,finished_at=?, \
                 error_code='lease_expired_local' WHERE id=? AND run_id=? AND step_instance_id=? \
                 AND execution_epoch=? AND state IN ('claimed','executing')",
            )
            .bind(now)
            .bind(attempt_id)
            .bind(&run_id)
            .bind(step_id)
            .bind(epoch)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
            db::require_one(
                attempt,
                "run-stale: local speech attempt changed during recovery",
            )?;
            let segment_state = if retry { "ready" } else { "failed" };
            sqlx::query(
                "UPDATE spaces_speech_segments SET state=?,state_reason='interrupted',updated_at=? \
                 WHERE id IN (SELECT segment_id FROM spaces_speech_renders \
                   WHERE run_attempt_id=? AND state='staged') AND state='rendering'",
            ).bind(segment_state).bind(now).bind(attempt_id)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
            sqlx::query(
                "UPDATE spaces_speech_renders SET state='failed',asset_path=NULL \
                 WHERE run_attempt_id=? AND state='staged'",
            )
            .bind(attempt_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
            let next_step_state = if retry { "ready" } else { "failed" };
            let step = sqlx::query(
                "UPDATE spaces_run_steps SET state=?,state_version=state_version+1,output=NULL, \
                 wait_reason=NULL,available_at=?,updated_at=? \
                 WHERE id=? AND run_id=? AND state='running'",
            )
            .bind(next_step_state)
            .bind(now)
            .bind(now)
            .bind(step_id)
            .bind(&run_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
            db::require_one(step, "run-stale: local speech step changed during recovery")?;
            super::super::append_event(
                tx,
                &run_id,
                Some(step_id),
                Some(attempt_id),
                if retry {
                    crate::spaces::runs::RunEventKind::StepRetryScheduled
                } else {
                    crate::spaces::runs::RunEventKind::StepFailed
                },
                &serde_json::json!({"reason":"lease_expired_local","executionEpoch":epoch + 1}),
                now,
            )
            .await?;
        }
        if !retry {
            sqlx::query(
                "UPDATE spaces_run_steps SET state='blocked',state_version=state_version+1,updated_at=? \
                 WHERE run_id=? AND state IN ('pending','ready')",
            ).bind(now).bind(&run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
        }
        let next_run_state = if retry { "queued" } else { "failed" };
        let run = sqlx::query(
            "UPDATE spaces_runs SET state=?,state_version=state_version+1, \
             execution_epoch=execution_epoch+1,wait_reason=NULL,updated_at=?,completed_at=? \
             WHERE id=? AND execution_epoch=? AND cancel_requested_at IS NULL AND completed_at IS NULL",
        ).bind(next_run_state).bind(now).bind((!retry).then_some(now)).bind(&run_id).bind(epoch)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
        db::require_one(run, "run-stale: local speech Run changed during recovery")?;
        recovered = recovered
            .checked_add(
                u64::try_from(attempts.len())
                    .map_err(|_| "Local speech recovery count overflow")?,
            )
            .ok_or("Local speech recovery count overflow")?;
    }
    Ok(recovered)
}

pub(super) async fn apply(pool: &SqlitePool) -> Result<bool, String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let pending: bool = sqlx::QueryBuilder::<sqlx::Sqlite>::new(CANCELLATION_TREE).push(
        " SELECT EXISTS(SELECT 1 FROM spaces_runs WHERE completed_at IS NULL
           AND (cancel_requested_at IS NOT NULL OR
             (pause_requested_at IS NOT NULL AND kind!='builtin_agent' AND state IN ('queued','running','waiting'))))
         OR EXISTS(SELECT 1 FROM spaces_runs WHERE completed_at IS NULL AND cancel_requested_at IS NULL
           AND created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER)*1000 <= $1)
         OR EXISTS(SELECT 1 FROM spaces_runs WHERE id IN (SELECT id FROM cancelled_tree)
           AND completed_at IS NULL AND cancel_requested_at IS NULL)",
    ).build_query_scalar().bind(now).fetch_one(pool).await.map_err(db::sql_error)?;
    if !pending {
        return Ok(false);
    }
    let mut tx = db::begin_write(pool).await?;
    propagate_cancellation(&mut tx, now).await?;
    let expired = expire_duration_budgets(&mut tx, now).await?;
    sqlx::query("UPDATE spaces_run_attempts SET state='cancelled',lease_until=NULL,finished_at=$1,error_code='run_cancelled' WHERE state IN ('claimed','executing','waiting_external') AND run_id IN (SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL)")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_run_steps SET state='cancelled',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE state IN ('pending','ready','running','waiting','paused','blocked') AND run_id IN (SELECT id FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL)")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    let cancelled = sqlx::query("UPDATE spaces_runs SET state='cancelled',state_version=state_version+1,updated_at=$1,completed_at=$1 WHERE cancel_requested_at IS NOT NULL AND completed_at IS NULL")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?.rows_affected();
    let paused = sqlx::query("UPDATE spaces_runs SET state='paused',state_version=state_version+1,wait_reason=NULL,updated_at=$1 WHERE pause_requested_at IS NOT NULL AND state IN ('queued','running','waiting') AND kind!='builtin_agent' AND NOT EXISTS (SELECT 1 FROM spaces_run_steps WHERE run_id=spaces_runs.id AND state='running')")
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?.rows_affected();
    tx.commit().await.map_err(db::sql_error)?;
    let changed = cancelled + paused + expired > 0;
    if changed {
        super::super::signals::announce();
    }
    Ok(changed)
}

async fn propagate_cancellation(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    now: i64,
) -> Result<(), String> {
    sqlx::QueryBuilder::<sqlx::Sqlite>::new(CANCELLATION_TREE)
        .push(
            " UPDATE spaces_runs SET cancel_requested_at=$1,
       execution_epoch=execution_epoch+1,updated_at=$1 WHERE id IN (SELECT id FROM cancelled_tree)
       AND cancel_requested_at IS NULL AND completed_at IS NULL",
        )
        .build()
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
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
    sqlx::QueryBuilder::<sqlx::Sqlite>::new(TREE)
        .push(
            " UPDATE spaces_run_attempts SET state='cancelled',lease_until=NULL,finished_at=$1,
         error_code=$3 WHERE state IN ('claimed','executing','waiting_external')
         AND run_id IN (SELECT id FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)
         AND completed_at IS NULL AND cancel_requested_at IS NULL)",
        )
        .build()
        .bind(now)
        .bind(root_id)
        .bind(error_code)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    sqlx::QueryBuilder::<sqlx::Sqlite>::new(TREE)
        .push(
            " UPDATE spaces_run_steps SET state='blocked',wait_reason='budget',
         state_version=state_version+1,updated_at=$1
         WHERE state IN ('pending','ready','running','waiting','paused','blocked')
         AND run_id IN (SELECT id FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)
         AND completed_at IS NULL AND cancel_requested_at IS NULL)",
        )
        .build()
        .bind(now)
        .bind(root_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let runs = sqlx::QueryBuilder::<sqlx::Sqlite>::new(TREE)
        .push(
            " UPDATE spaces_runs SET state='failed',wait_reason='budget',
         state_version=state_version+1,execution_epoch=execution_epoch+1,updated_at=$1,
         completed_at=$1 WHERE id IN (SELECT id FROM run_tree)
         AND completed_at IS NULL AND cancel_requested_at IS NULL",
        )
        .build()
        .bind(now)
        .bind(root_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(runs.rows_affected())
}

#[cfg(test)]
mod tests;
