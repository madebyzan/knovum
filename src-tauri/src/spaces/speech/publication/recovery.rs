use crate::spaces::{assets, db};
use std::{fs, path::Path};
use tauri::{AppHandle, Manager, Runtime};

pub async fn reconcile_interrupted<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    reconcile_previews(app)?;
    let pool = db::pool(app).await?;
    crate::spaces::speech::voice_candidates::reconcile_terminal_snapshots(app, pool).await?;
    crate::spaces::speech::asset_publication::reconcile(app, pool).await?;
    let disposable_audiobook_checkpoints: Vec<String> = sqlx::query_scalar(
        "SELECT c.project_revision_id FROM spaces_speech_chapter_checkpoints c \
         JOIN spaces_speech_project_revisions r ON r.id=c.project_revision_id \
         WHERE r.output_asset_path IS NOT NULL OR NOT EXISTS( \
           SELECT 1 FROM spaces_runs run WHERE run.completed_at IS NULL \
             AND json_extract(run.input_snapshot,'$.workflow')='speech_project_generation_v1' \
             AND json_extract(run.input_snapshot,'$.revisionId')=c.project_revision_id) \
         GROUP BY c.project_revision_id",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for revision_id in disposable_audiobook_checkpoints {
        crate::spaces::speech::project_publication::cleanup_audiobook_checkpoints(
            app,
            pool,
            &revision_id,
        )
        .await?;
    }
    sqlx::query(
        "DELETE FROM spaces_speech_loudness_checkpoints \
         WHERE EXISTS(SELECT 1 FROM spaces_speech_project_revisions r \
           WHERE r.id=spaces_speech_loudness_checkpoints.project_revision_id \
             AND r.output_asset_path IS NOT NULL) \
         OR NOT EXISTS(SELECT 1 FROM spaces_runs run WHERE run.completed_at IS NULL \
           AND json_extract(run.input_snapshot,'$.workflow')='speech_project_generation_v1' \
           AND json_extract(run.input_snapshot,'$.revisionId')=spaces_speech_loudness_checkpoints.project_revision_id)",
    ).execute(pool).await.map_err(db::sql_error)?;
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id,segment_id,asset_path FROM spaces_speech_renders WHERE state='staged'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (render_id, segment_id, candidate) in rows {
        if let Some(candidate) = candidate {
            let path = Path::new(&candidate);
            if path.exists() {
                let authorized = assets::authorize_spaces_asset(app, path)?;
                fs::remove_file(authorized).map_err(|error| error.to_string())?;
            }
            let part = path.with_extension("wav.part");
            if part.exists() {
                let authorized = assets::authorize_spaces_asset(app, &part)?;
                fs::remove_file(authorized).map_err(|error| error.to_string())?;
            }
        }
        let mut tx = db::begin_write(pool).await?;
        sqlx::query(
            "UPDATE spaces_speech_renders SET state='failed',asset_path=NULL WHERE id=? AND state='staged'",
        )
        .bind(&render_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        sqlx::query(
            "UPDATE spaces_speech_segments SET state='ready',state_reason='interrupted',updated_at=? \
             WHERE id=? AND state='rendering'",
        )
        .bind(crate::clock::unix_time_millis()?)
        .bind(&segment_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        tx.commit().await.map_err(db::sql_error)?;
    }
    crate::spaces::speech::video_publication::reconcile_workspaces(app)
}

fn reconcile_previews<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-previews");
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let managed = matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("wav") | Some("part")
        );
        if managed && path.is_file() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}
