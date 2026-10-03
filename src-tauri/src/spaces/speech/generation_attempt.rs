use super::contracts::{
    CachedSpeechRender, PresetVoiceExecution, SpeechSegmentExecution, SpeechSegmentRenderOutput,
};
use crate::spaces::db;

#[derive(Clone, Copy)]
pub(super) struct GenerationAttempt<'a> {
    pub project_id: &'a str,
    pub revision_id: &'a str,
    pub job_id: &'a str,
    pub epoch: i64,
    pub cancellation: &'a tokio_util::sync::CancellationToken,
}

pub(super) async fn begin_generation(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    job_id: &str,
) -> Result<i64, String> {
    uuid::Uuid::parse_str(project_id).map_err(|_| "Speech project identity is invalid")?;
    uuid::Uuid::parse_str(job_id).map_err(|_| "Speech job identity is invalid")?;
    sqlx::query_scalar(
        "UPDATE spaces_speech_projects SET generation_epoch=generation_epoch+1, \
         active_generation_job_id=?,generation_cancel_requested_at=NULL,updated_at=? \
         WHERE id=? AND active_generation_job_id IS NULL RETURNING generation_epoch",
    )
    .bind(job_id)
    .bind(crate::clock::unix_time_millis()?)
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Speech project already has an active generation attempt".to_owned())
}

pub(super) async fn finish_generation(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    job_id: &str,
    generation_epoch: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_speech_projects SET active_generation_job_id=NULL, \
         generation_cancel_requested_at=NULL,updated_at=? \
         WHERE id=? AND active_generation_job_id=? AND generation_epoch=?",
    )
    .bind(crate::clock::unix_time_millis()?)
    .bind(project_id)
    .bind(job_id)
    .bind(generation_epoch)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn clear_generation_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
    job_id: &str,
    generation_epoch: i64,
) -> Result<(), String> {
    let cleared = sqlx::query(
        "UPDATE spaces_speech_projects SET active_generation_job_id=NULL, \
         generation_cancel_requested_at=NULL,updated_at=? \
         WHERE id=? AND active_generation_job_id=? AND generation_epoch=? \
         AND generation_cancel_requested_at IS NULL",
    )
    .bind(crate::clock::unix_time_millis()?)
    .bind(project_id)
    .bind(job_id)
    .bind(generation_epoch)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(cleared, "Speech generation lost publication eligibility")
}

pub(super) async fn cached_render(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    segment_id: &str,
    input_digest: &str,
) -> Result<Option<CachedSpeechRender>, String> {
    let row: Option<CachedSpeechRender> = sqlx::query_as(
        "SELECT o.segment_id,o.asset_path AS path,o.output_sha256 AS sha256,o.sample_rate,o.sample_count \
         FROM spaces_speech_renders o JOIN spaces_speech_segments s ON s.id=o.segment_id \
         JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id \
         JOIN spaces_speech_projects p ON p.id=r.project_id \
         WHERE o.input_digest=? AND o.state='published' AND p.space_id=( \
           SELECT space_id FROM spaces_speech_projects WHERE id=? \
         ) ORDER BY (o.segment_id=?) DESC,o.published_at DESC LIMIT 1",
    )
    .bind(input_digest)
    .bind(project_id)
    .bind(segment_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(row)
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

pub(super) async fn link_cached_render(
    pool: &sqlx::SqlitePool,
    attempt: GenerationAttempt<'_>,
    segment: &SpeechSegmentExecution,
    voice: &PresetVoiceExecution,
    input_digest: &str,
    cached: &CachedSpeechRender,
    is_last: bool,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    if attempt.cancellation.is_cancelled() {
        return Err("cancelled".to_owned());
    }
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_speech_segments s \
         JOIN spaces_speech_projects p ON p.active_revision_id=s.project_revision_id \
         WHERE s.id=? AND s.project_revision_id=? AND s.state='ready' AND p.id=? \
         AND p.active_generation_job_id=? AND p.generation_epoch=? \
         AND p.generation_cancel_requested_at IS NULL)",
    )
    .bind(&segment.id)
    .bind(attempt.revision_id)
    .bind(attempt.project_id)
    .bind(attempt.job_id)
    .bind(attempt.epoch)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if !active {
        return Err("stale_revision: speech segment changed before cache publication".to_owned());
    }
    let attempt_epoch: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(attempt_epoch),-1)+1 FROM spaces_speech_renders WHERE segment_id=?",
    )
    .bind(&segment.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_speech_renders(id,segment_id,attempt_epoch,generation_job_id,generation_epoch, \
         input_digest,model_spec_id,engine_version,seed,synthesis_config,state,asset_path,output_sha256, \
         sample_rate,sample_count,created_at,published_at) \
         VALUES(?,?,?,?,?,?,?,?,0,?,'published',?,?,?,?,?,?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&segment.id)
    .bind(attempt_epoch)
    .bind(attempt.job_id)
    .bind(attempt.epoch)
    .bind(input_digest)
    .bind(&voice.model_spec_id)
    .bind(env!("CARGO_PKG_VERSION"))
    .bind(&voice.synthesis_config)
    .bind(&cached.path)
    .bind(&cached.sha256)
    .bind(cached.sample_rate)
    .bind(cached.sample_count)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let updated = sqlx::query(
        "UPDATE spaces_speech_segments SET state='rendered',state_reason=NULL,updated_at=? \
         WHERE id=? AND state='ready'",
    )
    .bind(now)
    .bind(&segment.id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(updated, "Speech segment changed before cache publication")?;
    if attempt.cancellation.is_cancelled() {
        return Err("cancelled".to_owned());
    }
    if is_last {
        clear_generation_in_transaction(&mut tx, attempt.project_id, attempt.job_id, attempt.epoch)
            .await?;
    }
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn mark_cached_segment(
    pool: &sqlx::SqlitePool,
    attempt: GenerationAttempt<'_>,
    segment_id: &str,
    is_last: bool,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    if attempt.cancellation.is_cancelled() {
        return Err("cancelled".to_owned());
    }
    let updated = sqlx::query(
        "UPDATE spaces_speech_segments SET state='rendered',state_reason=NULL,updated_at=? \
         WHERE id=? AND project_revision_id=? AND state IN ('ready','rendered') \
           AND EXISTS(SELECT 1 FROM spaces_speech_projects WHERE id=? AND active_revision_id=? \
             AND active_generation_job_id=? AND generation_epoch=? \
             AND generation_cancel_requested_at IS NULL)",
    )
    .bind(crate::clock::unix_time_millis()?)
    .bind(segment_id)
    .bind(attempt.revision_id)
    .bind(attempt.project_id)
    .bind(attempt.revision_id)
    .bind(attempt.job_id)
    .bind(attempt.epoch)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        updated,
        "stale_revision: speech segment changed before cache publication",
    )?;
    if attempt.cancellation.is_cancelled() {
        return Err("cancelled".to_owned());
    }
    if is_last {
        clear_generation_in_transaction(&mut tx, attempt.project_id, attempt.job_id, attempt.epoch)
            .await?;
    }
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn stage_render(
    pool: &sqlx::SqlitePool,
    attempt: GenerationAttempt<'_>,
    segment: &SpeechSegmentExecution,
    voice: &PresetVoiceExecution,
    input_digest: &str,
    target_path: &std::path::Path,
) -> Result<(String, u64), String> {
    let mut tx = db::begin_write(pool).await?;
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_speech_segments s \
         JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id \
         JOIN spaces_speech_projects p ON p.active_revision_id=r.id WHERE s.id=? AND p.id=? \
         AND p.active_generation_job_id=? AND p.generation_epoch=? \
         AND p.generation_cancel_requested_at IS NULL AND s.state='ready')",
    )
    .bind(&segment.id)
    .bind(attempt.project_id)
    .bind(attempt.job_id)
    .bind(attempt.epoch)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if !active {
        return Err("stale_revision: speech segment is no longer active".to_owned());
    }
    let attempt_epoch: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(attempt_epoch),-1)+1 FROM spaces_speech_renders WHERE segment_id=?",
    )
    .bind(&segment.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let render_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_speech_renders(id,segment_id,attempt_epoch,generation_job_id,generation_epoch, \
         input_digest,model_spec_id,engine_version,seed,synthesis_config,state,asset_path,created_at) \
         VALUES(?,?,?,?,?,?,?,?,0,?,'staged',?,?)",
    )
    .bind(&render_id)
    .bind(&segment.id)
    .bind(attempt_epoch)
    .bind(attempt.job_id)
    .bind(attempt.epoch)
    .bind(input_digest)
    .bind(&voice.model_spec_id)
    .bind(env!("CARGO_PKG_VERSION"))
    .bind(&voice.synthesis_config)
    .bind(target_path.to_string_lossy().as_ref())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let claimed = sqlx::query(
        "UPDATE spaces_speech_segments SET state='rendering',state_reason=NULL,updated_at=? \
         WHERE id=? AND state='ready'",
    )
    .bind(now)
    .bind(&segment.id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(claimed, "Speech segment changed before attempt claim")?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok((
        render_id,
        u64::try_from(attempt_epoch).map_err(|_| "Speech attempt epoch is invalid")?,
    ))
}

pub(super) async fn fail_render(
    pool: &sqlx::SqlitePool,
    render_id: &str,
    segment_id: &str,
    job_id: &str,
    generation_epoch: i64,
    error: &str,
    cancelled: bool,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let render_state = if cancelled { "cancelled" } else { "failed" };
    let segment_state = if cancelled { "ready" } else { "failed" };
    let failed = sqlx::query(
        "UPDATE spaces_speech_renders SET state=?,asset_path=NULL WHERE id=? AND state='staged' \
         AND generation_job_id=? AND generation_epoch=?",
    )
    .bind(render_state)
    .bind(render_id)
    .bind(job_id)
    .bind(generation_epoch)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if failed.rows_affected() == 1 {
        sqlx::query(
            "UPDATE spaces_speech_segments SET state=?,state_reason=?,updated_at=? WHERE id=?",
        )
        .bind(segment_state)
        .bind(error)
        .bind(crate::clock::unix_time_millis()?)
        .bind(segment_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    }
    tx.commit().await.map_err(db::sql_error)
}
