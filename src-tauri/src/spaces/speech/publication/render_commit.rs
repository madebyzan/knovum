use crate::spaces::{db, runs::AttemptClaim};
use std::{fs, path::Path};

pub(in crate::spaces::speech) async fn commit_render(
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    revision: crate::spaces::speech::contracts::ProjectRevisionIdentity<'_>,
    render_id: &str,
    segment_id: &str,
    path: &Path,
    audio: &crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<(), String> {
    let crate::spaces::speech::contracts::ProjectRevisionIdentity {
        project_id,
        revision_id,
    } = revision;
    let mut tx = db::begin_write(pool).await?;
    if let Err(error) = crate::spaces::speech::generation_attempt::require_active(
        &mut tx,
        claim,
        project_id,
        revision_id,
        segment_id,
    )
    .await
    {
        let rejected = sqlx::query(
            "UPDATE spaces_speech_renders SET state='rejected',asset_path=NULL \
             WHERE id=? AND state='staged' AND run_attempt_id=? AND run_execution_epoch=?",
        )
        .bind(render_id)
        .bind(&claim.attempt_id)
        .bind(claim.execution_epoch)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        if rejected.rows_affected() == 1 {
            sqlx::query(
                "UPDATE spaces_speech_segments SET state='ready',state_reason='stale_revision',updated_at=? \
                 WHERE id=? AND state='rendering'",
            )
            .bind(crate::foundation::clock::unix_time_millis()?).bind(segment_id)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
        }
        tx.commit().await.map_err(db::sql_error)?;
        let _ = fs::remove_file(path);
        return Err(error);
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    let published = sqlx::query(
        "UPDATE spaces_speech_renders SET state='published',asset_path=?,output_sha256=?, \
         sample_rate=?,sample_count=?,published_at=? WHERE id=? AND state='staged' \
         AND run_attempt_id=? AND run_execution_epoch=?",
    )
    .bind(path.to_string_lossy().as_ref())
    .bind(&audio.sha256)
    .bind(i64::from(audio.sample_rate))
    .bind(
        i64::try_from(audio.sample_count)
            .map_err(|_| "Speech sample count exceeds SQLite range")?,
    )
    .bind(now)
    .bind(render_id)
    .bind(&claim.attempt_id)
    .bind(claim.execution_epoch)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        published,
        "Speech render publication lost its staged attempt",
    )?;
    let segment = sqlx::query(
        "UPDATE spaces_speech_segments SET state='rendered',state_reason=NULL,updated_at=? \
         WHERE id=? AND state='rendering'",
    )
    .bind(now)
    .bind(segment_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(segment, "Speech segment changed before publication")?;
    crate::spaces::speech::asset_publication::remove_attempt_in_transaction(
        &mut tx,
        claim,
        path,
        &audio.sha256,
    )
    .await?;
    if let Err(error) = crate::spaces::speech::generation_attempt::require_active(
        &mut tx,
        claim,
        project_id,
        revision_id,
        segment_id,
    )
    .await
    {
        reject_cancelled(&mut tx, render_id, segment_id, claim).await?;
        tx.commit().await.map_err(db::sql_error)?;
        let _ = fs::remove_file(path);
        return Err(error);
    }
    match tx.commit().await {
        Ok(()) => Ok(()),
        Err(error) => {
            let published: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM spaces_speech_renders WHERE id=? AND state='published')",
            )
            .bind(render_id)
            .fetch_one(pool)
            .await
            .map_err(db::sql_error)?;
            if published {
                Ok(())
            } else {
                Err(db::sql_error(error))
            }
        }
    }
}

async fn reject_cancelled(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    render_id: &str,
    segment_id: &str,
    claim: &AttemptClaim,
) -> Result<(), String> {
    let cancelled = sqlx::query(
        "UPDATE spaces_speech_renders SET state='cancelled',asset_path=NULL,output_sha256=NULL, \
         sample_rate=NULL,sample_count=NULL,published_at=NULL \
         WHERE id=? AND state IN ('staged','published') \
         AND run_attempt_id=? AND run_execution_epoch=?",
    )
    .bind(render_id)
    .bind(&claim.attempt_id)
    .bind(claim.execution_epoch)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if cancelled.rows_affected() == 1 {
        sqlx::query(
            "UPDATE spaces_speech_segments SET state='ready',state_reason='cancelled',updated_at=? \
             WHERE id=? AND state='rendering'",
        )
        .bind(crate::foundation::clock::unix_time_millis()?).bind(segment_id)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}
