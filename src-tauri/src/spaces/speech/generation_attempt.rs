use super::contracts::{
    CachedSpeechRender, ProjectRevisionIdentity, SpeechSegmentExecution, SpeechSegmentRenderOutput,
    VoiceExecution,
};
use crate::spaces::{db, runs::AttemptClaim};

pub(super) async fn cached_render(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    segment_id: &str,
    input_digest: &str,
) -> Result<Option<CachedSpeechRender>, String> {
    sqlx::query_as(
        "SELECT o.id,o.asset_path AS path,o.output_sha256 AS sha256,o.sample_rate,o.sample_count          FROM spaces_speech_renders o JOIN spaces_speech_segments s ON s.id=o.segment_id          JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id          JOIN spaces_speech_projects p ON p.id=r.project_id          WHERE o.input_digest=? AND o.state='published' AND p.space_id=(            SELECT space_id FROM spaces_speech_projects WHERE id=?          ) ORDER BY (o.segment_id=?) DESC,o.published_at DESC LIMIT 1",
    )
    .bind(input_digest).bind(project_id).bind(segment_id)
    .fetch_optional(pool).await.map_err(db::sql_error)
}

pub(super) async fn reject_corrupt_cache(
    pool: &sqlx::SqlitePool,
    cached: &CachedSpeechRender,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_speech_renders SET state='rejected',asset_path=NULL,output_sha256=NULL, \
         sample_rate=NULL,sample_count=NULL,published_at=NULL \
         WHERE id=? AND state='published' AND output_sha256=?",
    )
    .bind(&cached.id)
    .bind(&cached.sha256)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(super) fn cached_output(
    segment: &SpeechSegmentExecution,
    row: &CachedSpeechRender,
) -> Result<SpeechSegmentRenderOutput, String> {
    Ok(SpeechSegmentRenderOutput {
        segment_id: segment.id.clone(),
        path: row.path.clone(),
        sha256: row.sha256.clone(),
        sample_rate: u32::try_from(row.sample_rate)
            .map_err(|_| "Cached speech sample rate is invalid")?,
        sample_count: u64::try_from(row.sample_count)
            .map_err(|_| "Cached speech sample count is invalid")?,
        cached: true,
    })
}

pub(super) async fn publish_cached(
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    revision: ProjectRevisionIdentity<'_>,
    segment: &SpeechSegmentExecution,
    voice: &VoiceExecution,
    input_digest: &str,
    cached: &CachedSpeechRender,
) -> Result<(), String> {
    let ProjectRevisionIdentity {
        project_id,
        revision_id,
    } = revision;
    let mut tx = db::begin_write(pool).await?;
    require_active(&mut tx, claim, project_id, revision_id, &segment.id).await?;
    let attempt_epoch = next_epoch(&mut tx, &segment.id).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_speech_renders(id,segment_id,attempt_epoch,run_id,run_step_id,run_attempt_id,run_execution_epoch,          input_digest,model_spec_id,engine_version,seed,synthesis_config,state,asset_path,output_sha256,          sample_rate,sample_count,created_at,published_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,'published',?,?,?,?,?,?)",
    )
    .bind(uuid::Uuid::new_v4().to_string()).bind(&segment.id).bind(attempt_epoch)
    .bind(&claim.run_id).bind(&claim.step_instance_id).bind(&claim.attempt_id).bind(claim.execution_epoch)
    .bind(input_digest).bind(&voice.model_spec_id).bind(env!("CARGO_PKG_VERSION"))
    .bind(i64::from(super::contracts::parse_synthesis_config(&serde_json::from_str(&voice.synthesis_config).map_err(db::json_error)?)?.seed)).bind(&voice.synthesis_config).bind(&cached.path).bind(&cached.sha256)
    .bind(cached.sample_rate).bind(cached.sample_count).bind(now).bind(now)
    .execute(&mut *tx).await.map_err(db::sql_error)?;
    mark_segment_rendered(&mut tx, &segment.id, now).await?;
    require_active(&mut tx, claim, project_id, revision_id, &segment.id).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn stage_render(
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    revision: ProjectRevisionIdentity<'_>,
    segment: &SpeechSegmentExecution,
    voice: &VoiceExecution,
    input_digest: &str,
    target_path: &std::path::Path,
) -> Result<(String, u64), String> {
    let ProjectRevisionIdentity {
        project_id,
        revision_id,
    } = revision;
    let mut tx = db::begin_write(pool).await?;
    require_active(&mut tx, claim, project_id, revision_id, &segment.id).await?;
    let attempt_epoch = next_epoch(&mut tx, &segment.id).await?;
    let render_id = uuid::Uuid::new_v4().to_string();
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_speech_renders(id,segment_id,attempt_epoch,run_id,run_step_id,run_attempt_id,run_execution_epoch,          input_digest,model_spec_id,engine_version,seed,synthesis_config,state,asset_path,created_at)          VALUES(?,?,?,?,?,?,?,?,?,?,?,?,'staged',?,?)",
    )
    .bind(&render_id).bind(&segment.id).bind(attempt_epoch).bind(&claim.run_id)
    .bind(&claim.step_instance_id).bind(&claim.attempt_id).bind(claim.execution_epoch)
    .bind(input_digest).bind(&voice.model_spec_id).bind(env!("CARGO_PKG_VERSION"))
    .bind(i64::from(super::contracts::parse_synthesis_config(&serde_json::from_str(&voice.synthesis_config).map_err(db::json_error)?)?.seed)).bind(&voice.synthesis_config).bind(target_path.to_string_lossy().as_ref()).bind(now)
    .execute(&mut *tx).await.map_err(db::sql_error)?;
    // The executing attempt is this segment's only writer: `require_active` already proved the
    // lease, the epoch and the active revision. The segment's own state is a report of its latest
    // attempt, so the claim overwrites it instead of filtering on it — a state left behind by an
    // interrupted attempt must never make a segment unrenderable.
    let claimed = sqlx::query(
        "UPDATE spaces_speech_segments SET state='rendering',state_reason=NULL,updated_at=? WHERE id=?",
    ).bind(now).bind(&segment.id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(claimed, "Speech segment no longer exists")?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok((
        render_id,
        u64::try_from(attempt_epoch).map_err(|_| "Speech attempt epoch is invalid")?,
    ))
}

pub(super) async fn fail_render(
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    render_id: &str,
    segment_id: &str,
    error: &str,
    cancelled: bool,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let render_state = if cancelled { "cancelled" } else { "failed" };
    let segment_state = if cancelled { "ready" } else { "failed" };
    let failed = sqlx::query(
        "UPDATE spaces_speech_renders SET state=?,asset_path=NULL WHERE id=? AND state='staged'          AND run_attempt_id=? AND run_execution_epoch=?",
    ).bind(render_state).bind(render_id).bind(&claim.attempt_id).bind(claim.execution_epoch)
    .execute(&mut *tx).await.map_err(db::sql_error)?;
    if failed.rows_affected() == 1 {
        sqlx::query(
            "UPDATE spaces_speech_segments SET state=?,state_reason=?,updated_at=? WHERE id=?",
        )
        .bind(segment_state)
        .bind(error)
        .bind(crate::foundation::clock::unix_time_millis()?)
        .bind(segment_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    }
    tx.commit().await.map_err(db::sql_error)
}

async fn next_epoch(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    segment_id: &str,
) -> Result<i64, String> {
    sqlx::query_scalar(
        "SELECT COALESCE(MAX(attempt_epoch),-1)+1 FROM spaces_speech_renders WHERE segment_id=?",
    )
    .bind(segment_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)
}

async fn mark_segment_rendered(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    segment_id: &str,
    now: i64,
) -> Result<(), String> {
    let update = sqlx::query(
        "UPDATE spaces_speech_segments SET state='rendered',state_reason=NULL,updated_at=? WHERE id=?",
    ).bind(now).bind(segment_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    db::require_one(update, "Speech segment changed before publication")
}

pub(super) async fn require_active(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    claim: &AttemptClaim,
    project_id: &str,
    revision_id: &str,
    segment_id: &str,
) -> Result<(), String> {
    if claim.cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_run_attempts a JOIN spaces_runs run ON run.id=a.run_id          JOIN spaces_speech_projects p ON p.id=? JOIN spaces_speech_segments s ON s.id=?          WHERE a.id=? AND a.run_id=? AND a.step_instance_id=? AND a.state='executing'            AND a.execution_epoch=? AND a.lease_until>? AND run.execution_epoch=a.execution_epoch            AND run.cancel_requested_at IS NULL AND run.completed_at IS NULL            AND p.active_revision_id=? AND s.project_revision_id=?)",
    )
    .bind(project_id).bind(segment_id).bind(&claim.attempt_id).bind(&claim.run_id)
    .bind(&claim.step_instance_id).bind(claim.execution_epoch).bind(crate::foundation::clock::unix_time_millis()?)
    .bind(revision_id).bind(revision_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if active {
        Ok(())
    } else {
        Err("attempt-stale: speech publication lease is no longer active".into())
    }
}
