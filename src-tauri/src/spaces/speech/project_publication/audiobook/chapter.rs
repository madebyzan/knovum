use std::fs;

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::{digest_render_order, part_path};
use super::{
    chapter_cache, chapter_commit, chapter_render, cleanup, ChapterCheckpoint,
    ChapterPublicationRequest,
};
use crate::spaces::{assets, runs::AttemptClaim};

pub(in crate::spaces::speech) async fn publish_chapter_checkpoint<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    request: &ChapterPublicationRequest<'_>,
    cancellation: &CancellationToken,
) -> Result<ChapterCheckpoint, String> {
    let digest = digest_render_order(
        request.renders,
        &serde_json::json!({
            "chapterKey": request.chapter_key,
            "sampleRate": request.sample_rate,
            "paragraphPauseMs": request.paragraph_pause_ms,
        }),
    );
    let directory = assets::asset_directory(app)?
        .join("speech")
        .join(request.project_id)
        .join(request.revision_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let work = directory.join(format!("chapter-assembly-{}", claim.attempt_id));
    if work.exists() {
        fs::remove_dir_all(&work).map_err(|error| error.to_string())?;
    }
    if let Some(checkpoint) = chapter_cache::load(
        app,
        pool,
        &directory,
        request.revision_id,
        request.chapter_key,
        &digest,
        request.sample_rate,
    )
    .await?
    {
        return Ok(checkpoint);
    }
    fs::create_dir_all(&work).map_err(|error| error.to_string())?;
    let output = directory.join(format!("chapter-{digest}-{}.wav", claim.attempt_id));
    let part = part_path(&output);
    let rendered =
        match chapter_render::render(app, request, output, part.clone(), &work, cancellation).await
        {
            Ok(rendered) => rendered,
            Err(error) => return Err(cleanup::paths(error, &[&part], &[&work])),
        };
    let result = chapter_commit::commit(app, pool, claim, request, &digest, &rendered, &work).await;
    if result.is_ok() {
        fs::remove_dir_all(&work).map_err(|error| error.to_string())?;
    }
    result
}
