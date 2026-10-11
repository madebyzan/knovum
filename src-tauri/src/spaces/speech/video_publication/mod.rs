use std::fs;

mod asset_transaction;
mod dialogue_envelope;
pub(in crate::spaces::speech) mod final_mix;
pub(in crate::spaces::speech) mod mixing;
pub(in crate::spaces::speech) mod policy;
mod recovery;
#[cfg(test)]
mod tests;
mod video_mux;
mod voice_lanes;

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput};
use crate::{
    file_inspection::verify_content_revision,
    local_resources::tools,
    spaces::{assets, runs::AttemptClaim},
};

const SAMPLE_RATE: u32 = 48_000;
const MAX_MIX_INPUTS: usize = 16;

pub(super) struct VideoPublicationInput<'a> {
    pub(super) project_id: &'a str,
    pub(super) revision_id: &'a str,
    pub(super) output_config: &'a serde_json::Value,
    pub(super) windows: &'a [(Option<i64>, Option<i64>)],
    pub(super) renders: &'a [SpeechSegmentRenderOutput],
}

pub(super) use recovery::reconcile_workspaces;

pub(super) async fn publish_video<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    input: VideoPublicationInput<'_>,
    cancellation: &CancellationToken,
) -> Result<SpeechAudioOutput, String> {
    let VideoPublicationInput {
        project_id,
        revision_id,
        output_config,
        windows,
        renders,
    } = input;
    uuid::Uuid::parse_str(project_id).map_err(|_| "Speech project identity is invalid")?;
    uuid::Uuid::parse_str(revision_id).map_err(|_| "Speech revision identity is invalid")?;
    uuid::Uuid::parse_str(&claim.attempt_id)
        .map_err(|_| "Speech assembly attempt identity is invalid")?;
    if renders.is_empty() || windows.len() != renders.len() {
        return Err("Video dubbing has no complete timed render set".into());
    }
    let config = super::video_config::parse(output_config)?;
    let digest = policy::assembly_digest(renders, windows, output_config);
    let directory = assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let video =
        policy::verified_source(app, &config.video.asset_path, &config.video.source_revision)?;
    let background = config
        .background
        .as_ref()
        .map(|source| policy::verified_source(app, &source.asset_path, &source.source_revision))
        .transpose()?;
    let postcheck_video = video.clone();
    let postcheck_background = background.clone();
    let video_revision = config.video.source_revision.clone();
    let background_revision = config
        .background
        .as_ref()
        .map(|source| source.source_revision.clone());
    let ffmpeg = tools::resolve_ffmpeg(app)?.ok_or("The bundled FFmpeg tool is unavailable")?;
    let inspect_ffmpeg = ffmpeg.clone();
    let inspect_video = video.clone();
    let expected_video_ms = config.video.duration_ms;
    let inspect_cancellation = cancellation.clone();
    let video_duration_us = crate::foundation::blocking::run(move || {
        policy::verified_duration_us(
            &inspect_ffmpeg,
            &inspect_video,
            "0:v:0",
            expected_video_ms,
            &inspect_cancellation,
        )
    })
    .await?;
    if let (Some(source), Some(path)) = (config.background.as_ref(), background.as_ref()) {
        let inspect_ffmpeg = ffmpeg.clone();
        let inspect_path = path.clone();
        let expected_ms = source.duration_ms;
        let inspect_cancellation = cancellation.clone();
        crate::foundation::blocking::run(move || {
            policy::verified_duration_us(
                &inspect_ffmpeg,
                &inspect_path,
                "0:a:0",
                expected_ms,
                &inspect_cancellation,
            )
        })
        .await?;
    }
    let video_samples = policy::samples_for_duration(video_duration_us)?;
    if let Some(output) =
        recovery::reusable_output(app, pool, project_id, revision_id, &digest, video_samples)
            .await?
    {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        verify_content_revision(&postcheck_video, &video_revision)?;
        if let (Some(path), Some(revision)) = (
            postcheck_background.as_deref(),
            background_revision.as_deref(),
        ) {
            verify_content_revision(path, revision)?;
        }
        return Ok(output);
    }
    let timed = mixing::timed_renders(renders, windows, video_samples)?;
    let work = directory.join(format!("video-assembly-{}", claim.attempt_id));
    if work.exists() {
        fs::remove_dir_all(&work).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&work).map_err(|error| error.to_string())?;
    let build_ffmpeg = ffmpeg.clone();
    let build_work = work.clone();
    let build_background = background;
    let build_config = config.clone();
    let build_cancellation = cancellation.clone();
    let mixed = crate::foundation::blocking::run(move || {
        mixing::build_mix(
            &build_ffmpeg,
            &build_work,
            &timed,
            video_samples,
            build_background.as_deref(),
            &build_config,
            &build_cancellation,
        )
    })
    .await;
    let mixed = match mixed {
        Ok(path) => path,
        Err(error) => {
            return Err(asset_transaction::cleanup_failure(error, &[], &[&work]));
        }
    };
    asset_transaction::publish(asset_transaction::Publication {
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
        retain_original_track: config.retain_original_track,
        cancellation,
    })
    .await
}
