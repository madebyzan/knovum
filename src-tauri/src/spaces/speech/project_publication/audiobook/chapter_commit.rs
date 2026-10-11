use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};

use super::{
    chapter_render::RenderedChapter, cleanup, ChapterCheckpoint, ChapterPublicationRequest,
};
use crate::spaces::speech::{asset_publication, publication::hash_file};
use crate::spaces::{db, runs::AttemptClaim};

pub(super) async fn commit<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    request: &ChapterPublicationRequest<'_>,
    digest: &str,
    rendered: &RenderedChapter,
    work: &Path,
) -> Result<ChapterCheckpoint, String> {
    let publication_id = match asset_publication::stage_for_attempt(
        pool,
        claim,
        asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::ChapterCheckpoint,
            owner_id: request.revision_id,
            asset_path: &rendered.output,
            part_path: Some(&rendered.part),
            sha256: &rendered.sha256,
        },
    )
    .await
    {
        Ok(id) => id,
        Err(error) => {
            return Err(cleanup::paths(
                error,
                &[&rendered.part, &rendered.output],
                &[work],
            ))
        }
    };
    let mut commit_outcome_uncertain = false;
    let publication: Result<ChapterCheckpoint, String> = async {
        fs::rename(&rendered.part, &rendered.output).map_err(|error| error.to_string())?;
        if hash_file(&rendered.output)? != rendered.sha256 {
            return Err("Audiobook chapter hash changed during atomic publication".into());
        }
        let mut tx = db::begin_write(pool).await?;
        let inserted = sqlx::query(
            "INSERT INTO spaces_speech_chapter_checkpoints( \
               id,project_revision_id,chapter_key,input_digest,asset_path,sha256,sample_rate,sample_count,created_at) \
             SELECT ?,?,?,?,?,?,?,?,? WHERE EXISTS( \
               SELECT 1 FROM spaces_run_attempts a JOIN spaces_runs r ON r.id=a.run_id \
               WHERE a.id=? AND a.run_id=? AND a.step_instance_id=? AND a.state='executing' \
                 AND a.execution_epoch=? AND a.lease_until>? AND r.execution_epoch=a.execution_epoch \
                 AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(request.revision_id)
        .bind(request.chapter_key)
        .bind(digest)
        .bind(rendered.output.to_string_lossy().as_ref())
        .bind(&rendered.sha256)
        .bind(i64::from(request.sample_rate))
        .bind(i64::try_from(rendered.sample_count).map_err(|_| "Chapter sample count exceeds SQLite range")?)
        .bind(crate::foundation::clock::unix_time_millis()?)
        .bind(&claim.attempt_id)
        .bind(&claim.run_id)
        .bind(&claim.step_instance_id)
        .bind(claim.execution_epoch)
        .bind(crate::foundation::clock::unix_time_millis()?)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        db::require_one(inserted, "attempt-stale: chapter publication lease expired")?;
        asset_publication::remove_in_transaction(&mut tx, &publication_id).await?;
        if let Err(commit_error) = tx.commit().await {
            let committed = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM spaces_speech_chapter_checkpoints \
                 WHERE project_revision_id=? AND chapter_key=? AND input_digest=? \
                   AND asset_path=? AND sha256=?)",
            )
            .bind(request.revision_id)
            .bind(request.chapter_key)
            .bind(digest)
            .bind(rendered.output.to_string_lossy().as_ref())
            .bind(&rendered.sha256)
            .fetch_one(pool)
            .await;
            match committed {
                Ok(true) => {}
                Ok(false) => return Err(db::sql_error(commit_error)),
                Err(verification_error) => {
                    commit_outcome_uncertain = true;
                    return Err(format!(
                        "{}; chapter publication commit outcome could not be confirmed: {}",
                        db::sql_error(commit_error),
                        db::sql_error(verification_error),
                    ));
                }
            }
        }
        Ok(ChapterCheckpoint {
            chapter_key: request.chapter_key.into(),
            path: rendered.output.to_string_lossy().into_owned(),
            sha256: rendered.sha256.clone(),
            sample_rate: request.sample_rate,
            sample_count: rendered.sample_count,
        })
    }
    .await;
    match publication {
        Ok(checkpoint) => Ok(checkpoint),
        Err(mut error) => {
            if !commit_outcome_uncertain {
                if let Err(cleanup_error) =
                    asset_publication::abandon(app, pool, &publication_id).await
                {
                    error.push_str("; failed to abandon chapter publication: ");
                    error.push_str(&cleanup_error);
                }
                Err(cleanup::paths(
                    error,
                    &[&rendered.part, &rendered.output],
                    &[work],
                ))
            } else {
                Err(cleanup::paths(error, &[&rendered.part], &[work]))
            }
        }
    }
}
