use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    super::{
        source_separation::media::{
            budget_ms, extract_audio, hash_file, pcm_wav_metadata, SAMPLE_RATE,
        },
        video_config::SpeechMediaSource,
    },
    policy,
};
use crate::{
    ai::{
        contracts::speech_wire::{
            AudioFileReference, InferenceRequestIdentity, SpeechSeparateParams,
        },
        local_runtime::{admission::Priority, device_profiles, separation},
    },
    file_inspection::verify_content_revision,
    local_resources::{model_selection, preparation},
    spaces::db,
};

pub(in crate::spaces::speech) struct AutoBackgroundContext<'a, R: Runtime> {
    pub app: &'a AppHandle<R>,
    pub pool: &'a SqlitePool,
    pub project_id: &'a str,
    pub job_id: &'a str,
    pub attempt_epoch: u64,
    pub ffmpeg: &'a Path,
    pub video_path: &'a Path,
    pub video_source: &'a SpeechMediaSource,
    pub work: &'a Path,
    pub cancellation: &'a CancellationToken,
}

pub(in crate::spaces::speech) async fn resolve<R: Runtime>(
    ctx: AutoBackgroundContext<'_, R>,
) -> Result<Option<PathBuf>, String> {
    if let Some(published) = reusable_published_separation(&ctx).await? {
        return Ok(Some(published));
    }
    let cache_path = cached_background_path(ctx.app, ctx.project_id, ctx.video_source)?;
    if cache_path.is_file() && pcm_wav_metadata(&cache_path).is_ok() {
        return Ok(Some(cache_path));
    }
    let check_ffmpeg = ctx.ffmpeg.to_owned();
    let check_video = ctx.video_path.to_owned();
    let check_cancel = ctx.cancellation.clone();
    let has_audio = crate::foundation::blocking::run(move || {
        Ok::<bool, String>(
            crate::media::ffmpeg::duration_us(&check_ffmpeg, &check_video, "0:a:0", &check_cancel)
                .is_ok(),
        )
    })
    .await?;
    if !has_audio {
        return Ok(None);
    }
    separate_and_cache(&ctx, &cache_path).await.map(Some)
}

async fn reusable_published_separation<R: Runtime>(
    ctx: &AutoBackgroundContext<'_, R>,
) -> Result<Option<PathBuf>, String> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT s.background_asset_path, s.background_sha256 \
         FROM spaces_speech_source_separations s \
         JOIN spaces_speech_projects p ON p.space_id = s.space_id \
         WHERE p.id = ? AND s.source_asset_path = ? AND s.source_revision = ? \
         ORDER BY s.created_at DESC LIMIT 1",
    )
    .bind(ctx.project_id)
    .bind(&ctx.video_source.asset_path)
    .bind(&ctx.video_source.source_revision)
    .fetch_optional(ctx.pool)
    .await
    .map_err(db::sql_error)?;
    let Some((asset_path, sha256)) = row else {
        return Ok(None);
    };
    let revision = format!("sha256:{sha256}");
    Ok(policy::verified_source(ctx.app, &asset_path, &revision).ok())
}

fn cached_background_path<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    video_source: &SpeechMediaSource,
) -> Result<PathBuf, String> {
    let digest = video_source
        .source_revision
        .strip_prefix("sha256:")
        .ok_or("Video source revision is invalid")?;
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-auto-backgrounds");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join(format!("{project_id}-{digest}.wav")))
}

async fn separate_and_cache<R: Runtime>(
    ctx: &AutoBackgroundContext<'_, R>,
    cache_path: &Path,
) -> Result<PathBuf, String> {
    let model = model_selection::speech_separation(ctx.app)?;
    let device_profile = device_profiles::speech_separation(model)?;
    let spec_id = model.separation_spec()?.to_owned();
    let extracted = ctx.work.join("auto-source.wav");
    let vocals = ctx.work.join("auto-vocals.wav");
    let background = ctx.work.join("auto-background.wav");
    let extract_ffmpeg = ctx.ffmpeg.to_owned();
    let extract_video = ctx.video_path.to_owned();
    let extract_target = extracted.clone();
    let extract_cancel = ctx.cancellation.clone();
    let video_revision = ctx.video_source.source_revision.clone();
    let duration_ms = ctx.video_source.duration_ms;
    let (byte_length, sample_count, sha256) =
        crate::foundation::blocking::run(move || -> Result<_, String> {
            extract_audio(
                &extract_ffmpeg,
                &extract_video,
                &extract_target,
                &extract_cancel,
            )?;
            verify_content_revision(&extract_video, &video_revision)?;
            let (byte_length, sample_count) = pcm_wav_metadata(&extract_target)?;
            let deadline = Instant::now() + Duration::from_millis(budget_ms(duration_ms));
            let sha256 = hash_file(&extract_target, &extract_cancel, deadline)?;
            Ok((byte_length, sample_count, sha256))
        })
        .await?;
    let models = preparation::prepare_models_for_job(
        ctx.app,
        model.preparation(),
        &format!("{}:auto-separation", ctx.job_id),
        "Preparing background separation",
        ctx.cancellation,
        &mut |_| {},
    )
    .await?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let separated = separation::separate_to_paths(
        ctx.app,
        &models,
        device_profile,
        SpeechSeparateParams {
            identity: InferenceRequestIdentity {
                request_id,
                owner_kind: "video_dubbing_auto_separation".into(),
                owner_id: ctx.project_id.to_owned(),
                attempt_id: ctx.job_id.to_owned(),
                attempt_epoch: ctx.attempt_epoch,
                input_revision: ctx.video_source.source_revision.clone(),
                model_spec_id: spec_id,
                budget_ms: budget_ms(duration_ms),
            },
            audio_ref: AudioFileReference {
                path: extracted.to_string_lossy().into_owned(),
                media_type: "audio/wav".into(),
                byte_length,
                sha256,
                sample_rate: SAMPLE_RATE,
                channels: 2,
                sample_format: "pcm_s16le".into(),
            },
        },
        ctx.cancellation,
        Priority::Background,
        separation::SeparationTargets {
            vocals: &vocals,
            background: &background,
        },
    )
    .await?;
    let _ = fs::remove_file(&vocals);
    let _ = fs::remove_file(&extracted);
    if separated.background.sample_count != sample_count {
        let _ = fs::remove_file(&background);
        return Err("invalid_output: separated background timeline differs from its source".into());
    }
    let cache_part = ctx.work.join("auto-background.cache.part");
    if fs::copy(&background, &cache_part).is_ok() && fs::rename(&cache_part, cache_path).is_ok() {
        return Ok(cache_path.to_owned());
    }
    let _ = fs::remove_file(&cache_part);
    Ok(background)
}
