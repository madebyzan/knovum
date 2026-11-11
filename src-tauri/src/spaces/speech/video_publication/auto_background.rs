//! The background a dub uses when it is separated from the video itself.
//!
//! A published separation of the same video by the same model is reused first; then a background
//! this project already separated with that model; otherwise the video is separated now and the
//! background kept for the next assembly. Every candidate is identified by the source revision and
//! the separation model, so changing either never reuses a background it did not produce.

use std::{
    fs,
    path::{Path, PathBuf},
};

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    super::{
        source_separation::{
            media::pcm_wav_metadata,
            separate::{self, Requester, Separation},
        },
        video_config::SpeechMediaSource,
    },
    policy,
};
use crate::{
    ai::local_runtime::separation::SeparationTargets, local_resources::model_catalog::LocalModelId,
    spaces::db,
};

pub(in crate::spaces::speech) struct AutoBackgroundContext<'a, R: Runtime> {
    pub app: &'a AppHandle<R>,
    pub pool: &'a SqlitePool,
    pub model: LocalModelId,
    pub project_id: &'a str,
    pub job_id: &'a str,
    pub attempt_epoch: u64,
    pub ffmpeg: &'a Path,
    pub video_path: &'a Path,
    pub video_source: &'a SpeechMediaSource,
    pub work: &'a Path,
    pub cancellation: &'a CancellationToken,
}

/// The separated background of the video, or `None` when the video has no audio to separate.
pub(in crate::spaces::speech) async fn resolve<R: Runtime>(
    ctx: AutoBackgroundContext<'_, R>,
) -> Result<Option<PathBuf>, String> {
    let model_spec_id = ctx.model.separation_spec()?;
    if let Some(published) = published_background(&ctx, model_spec_id).await? {
        return Ok(Some(published));
    }
    let cached = cached_background_path(&ctx, model_spec_id)?;
    if cached.is_file() {
        // Entries are only ever renamed into place whole, so one that does not read as the
        // separation format was damaged outside Knovum and is separated again.
        if pcm_wav_metadata(&cached).is_ok() {
            return Ok(Some(cached));
        }
        fs::remove_file(&cached).map_err(|error| error.to_string())?;
    }
    if !has_audio(&ctx).await? {
        return Ok(None);
    }
    separate_into_cache(&ctx, &cached).await.map(Some)
}

/// The newest published separation of this exact video by this model whose background is still
/// the file it published. A background edited or removed since is not this separation any more.
async fn published_background<R: Runtime>(
    ctx: &AutoBackgroundContext<'_, R>,
    model_spec_id: &str,
) -> Result<Option<PathBuf>, String> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT s.background_asset_path, s.background_sha256 \
         FROM spaces_speech_source_separations s \
         JOIN spaces_speech_projects p ON p.space_id = s.space_id \
         WHERE p.id = ? AND s.source_asset_path = ? AND s.source_revision = ? \
         AND s.model_spec_id = ? ORDER BY s.created_at DESC",
    )
    .bind(ctx.project_id)
    .bind(&ctx.video_source.asset_path)
    .bind(&ctx.video_source.source_revision)
    .bind(model_spec_id)
    .fetch_all(ctx.pool)
    .await
    .map_err(db::sql_error)?;
    for (asset_path, sha256) in rows {
        let revision = format!("sha256:{sha256}");
        if let Ok(path) = policy::verified_source(ctx.app, &asset_path, &revision) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn cached_background_path<R: Runtime>(
    ctx: &AutoBackgroundContext<'_, R>,
    model_spec_id: &str,
) -> Result<PathBuf, String> {
    let digest = ctx
        .video_source
        .source_revision
        .strip_prefix("sha256:")
        .ok_or("Video source revision is invalid")?;
    let directory = ctx
        .app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-auto-backgrounds");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join(format!("{}-{digest}-{model_spec_id}.wav", ctx.project_id)))
}

async fn has_audio<R: Runtime>(ctx: &AutoBackgroundContext<'_, R>) -> Result<bool, String> {
    let ffmpeg = ctx.ffmpeg.to_owned();
    let video = ctx.video_path.to_owned();
    let cancellation = ctx.cancellation.clone();
    crate::foundation::blocking::run(move || {
        crate::media::ffmpeg::has_audio(&ffmpeg, &video, &cancellation, "cancelled")
    })
    .await
}

async fn separate_into_cache<R: Runtime>(
    ctx: &AutoBackgroundContext<'_, R>,
    cached: &Path,
) -> Result<PathBuf, String> {
    let vocals = ctx.work.join("auto-vocals.wav");
    let background = ctx.work.join("auto-background.wav");
    let request_id = uuid::Uuid::new_v4().to_string();
    separate::separate(Separation {
        app: ctx.app,
        model: ctx.model,
        path: ctx.video_path,
        source: ctx.video_source,
        requester: Requester {
            request_id: &request_id,
            owner_kind: "video_dubbing_auto_separation",
            owner_id: ctx.project_id,
            attempt_id: ctx.job_id,
            attempt_epoch: ctx.attempt_epoch,
        },
        workspace: ctx.work,
        targets: SeparationTargets {
            vocals: &vocals,
            background: &background,
        },
        cancellation: ctx.cancellation,
    })
    .await?;
    fs::remove_file(&vocals).map_err(|error| error.to_string())?;
    let part = cached.with_extension("wav.part");
    fs::copy(&background, &part).map_err(|error| error.to_string())?;
    fs::rename(&part, cached).map_err(|error| {
        let _ = fs::remove_file(&part);
        error.to_string()
    })?;
    fs::remove_file(&background).map_err(|error| error.to_string())?;
    Ok(cached.to_owned())
}
