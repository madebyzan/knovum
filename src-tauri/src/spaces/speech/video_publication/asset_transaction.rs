use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::{contracts::SpeechAudioOutput, publication::hash_file};
use super::SAMPLE_RATE;
use crate::{file_inspection::verify_content_revision, spaces::runs::AttemptClaim};

pub(super) struct Publication<'a, R: Runtime> {
    pub app: &'a AppHandle<R>,
    pub pool: &'a sqlx::SqlitePool,
    pub claim: &'a AttemptClaim,
    pub revision_id: &'a str,
    pub directory: PathBuf,
    pub digest: String,
    pub ffmpeg: PathBuf,
    pub video: PathBuf,
    pub mixed: PathBuf,
    pub work: PathBuf,
    pub postcheck_video: PathBuf,
    pub postcheck_background: Option<PathBuf>,
    pub video_revision: String,
    pub background_revision: Option<String>,
    pub video_duration_us: u64,
    pub video_samples: u64,
    pub cancellation: &'a CancellationToken,
}

pub(super) async fn publish<R: Runtime>(
    request: Publication<'_, R>,
) -> Result<SpeechAudioOutput, String> {
    let Publication {
        app,
        pool,
        claim,
        revision_id,
        directory,
        digest,
        ffmpeg,
        video,
        mixed,
        work,
        postcheck_video,
        postcheck_background,
        video_revision,
        background_revision,
        video_duration_us,
        video_samples,
        cancellation,
    } = request;
    let output = directory.join(format!("full-{digest}-{}.mp4", claim.attempt_id));
    let part = output.with_extension("mp4.part");
    if part.exists() {
        if let Err(error) = fs::remove_file(&part) {
            return Err(cleanup_failure(error.to_string(), &[], &[&work]));
        }
    }
    let verify_ffmpeg = ffmpeg.clone();
    let mux_ffmpeg = ffmpeg;
    let mux_source = video;
    let mux_audio = mixed;
    let mux_part = part.clone();
    let mux_cancellation = cancellation.clone();
    let muxed = crate::blocking::run(move || {
        super::media_pipeline::mux_video(
            &mux_ffmpeg,
            &mux_source,
            &mux_audio,
            &mux_part,
            &mux_cancellation,
        )
    })
    .await;
    if let Err(error) = muxed {
        return Err(cleanup_failure(error, &[&part], &[&work]));
    }
    let verify_part = part.clone();
    let verify_cancellation = cancellation.clone();
    let postcheck = crate::blocking::run(move || {
        verify_content_revision(&postcheck_video, &video_revision)?;
        if let (Some(path), Some(revision)) = (
            postcheck_background.as_deref(),
            background_revision.as_deref(),
        ) {
            verify_content_revision(path, revision)?;
        }
        let final_video_us = super::media_pipeline::inspect_duration_us(
            &verify_ffmpeg,
            &verify_part,
            "0:v:0",
            &verify_cancellation,
        )?;
        if final_video_us.abs_diff(video_duration_us) > 2_000 {
            return Err("Published video does not preserve the verified source EOF".into());
        }
        let final_audio_us = super::media_pipeline::inspect_duration_us(
            &verify_ffmpeg,
            &verify_part,
            "0:a:0",
            &verify_cancellation,
        )?;
        if final_audio_us.abs_diff(video_duration_us) > 25_000 {
            return Err("Published video audio does not reach the source video EOF".into());
        }
        Ok(())
    })
    .await;
    if let Err(error) = postcheck {
        return Err(cleanup_failure(error, &[&part], &[&work]));
    }
    if cancellation.is_cancelled() {
        return Err(cleanup_failure("cancelled".into(), &[&part], &[&work]));
    }
    let sha256 = match hash_file(&part) {
        Ok(hash) => hash,
        Err(error) => return Err(cleanup_failure(error, &[&part], &[&work])),
    };
    let publication_id = super::super::asset_publication::stage_for_attempt(
        pool,
        claim,
        super::super::asset_publication::PendingAsset {
            owner_kind: "project_output",
            owner_id: revision_id,
            asset_path: &output,
            part_path: Some(&part),
            sha256: &sha256,
        },
    )
    .await
    .map_err(|error| cleanup_failure(error, &[&part], &[&work]))?;
    if let Err(error) = fs::rename(&part, &output) {
        let error = cleanup_failure(error.to_string(), &[&part], &[&work]);
        super::super::asset_publication::abandon(app, pool, &publication_id).await?;
        return Err(error);
    }
    let published_hash = match hash_file(&output) {
        Ok(hash) => hash,
        Err(error) => {
            let error = cleanup_failure(error, &[&output], &[&work]);
            super::super::asset_publication::abandon(app, pool, &publication_id).await?;
            return Err(error);
        }
    };
    if published_hash != sha256 {
        let error = cleanup_failure(
            "Published video hash changed during atomic publication".into(),
            &[&output],
            &[&work],
        );
        super::super::asset_publication::abandon(app, pool, &publication_id).await?;
        return Err(error);
    }
    if let Err(error) = fs::remove_dir_all(work) {
        let error = cleanup_failure(error.to_string(), &[&output], &[]);
        super::super::asset_publication::abandon(app, pool, &publication_id).await?;
        return Err(error);
    }
    Ok(SpeechAudioOutput {
        path: output.to_string_lossy().into_owned(),
        sha256,
        sample_rate: SAMPLE_RATE,
        sample_count: video_samples,
    })
}

pub(super) fn cleanup_failure(mut cause: String, files: &[&Path], directories: &[&Path]) -> String {
    for path in files {
        if let Err(error) = fs::remove_file(path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                cause.push_str(&format!("; failed to remove {}: {error}", path.display()));
            }
        }
    }
    for path in directories {
        if let Err(error) = fs::remove_dir_all(path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                cause.push_str(&format!("; failed to remove {}: {error}", path.display()));
            }
        }
    }
    cause
}
