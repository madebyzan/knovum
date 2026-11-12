//! The one path from a media source to its separated vocals and background.
//!
//! A source-separation Run and a dub's automatic background both separate through here: decode the
//! source's first audio stream, confirm the source did not change while it was decoded, prepare the
//! chosen model, run it on the backend the runtime reports, and confirm the stems cover exactly the
//! decoded timeline.

use std::path::Path;

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::media::{budget_ms, extract_audio, hash_file, pcm_wav_metadata, SAMPLE_RATE};
use crate::{
    ai::{
        contracts::speech_wire::{
            AudioFileReference, InferenceRequestIdentity, SpeechSeparateParams,
        },
        local_runtime::{
            admission::Priority,
            device_profiles,
            separation::{self, SeparatedAudio, SeparationTargets},
        },
    },
    file_inspection::verify_content_revision,
    local_resources::{model_catalog::LocalModelId, preparation, runtimes, tools},
    spaces::speech::video_config::SpeechMediaSource,
};

/// Who asked for a separation, as its inference request records it.
pub(in crate::spaces::speech) struct Requester<'a> {
    pub request_id: &'a str,
    pub owner_kind: &'static str,
    pub owner_id: &'a str,
    pub attempt_id: &'a str,
    pub attempt_epoch: u64,
}

pub(in crate::spaces::speech) struct Separation<'a, R: Runtime> {
    pub app: &'a AppHandle<R>,
    pub model: LocalModelId,
    /// The authorized file `source` describes.
    pub path: &'a Path,
    pub source: &'a SpeechMediaSource,
    pub requester: Requester<'a>,
    /// A private directory the decoded source is staged in.
    pub workspace: &'a Path,
    pub targets: SeparationTargets<'a>,
    pub cancellation: &'a CancellationToken,
}

pub(in crate::spaces::speech) async fn separate<R: Runtime>(
    job: Separation<'_, R>,
) -> Result<SeparatedAudio, String> {
    let ffmpeg = tools::resolve_ffmpeg(job.app)?.ok_or("FFmpeg runtime is unavailable")?;
    let decoded = job.workspace.join("separation-source.wav");
    let (byte_length, sample_count, sha256) = crate::foundation::blocking::run({
        let (path, decoded) = (job.path.to_owned(), decoded.clone());
        let (revision, duration_ms) = (job.source.source_revision.clone(), job.source.duration_ms);
        let cancellation = job.cancellation.clone();
        move || -> Result<_, String> {
            extract_audio(&ffmpeg, &path, &decoded, &cancellation)?;
            verify_content_revision(&path, &revision)?;
            let (byte_length, sample_count) = pcm_wav_metadata(&decoded)?;
            let deadline = std::time::Instant::now()
                + std::time::Duration::from_millis(budget_ms(duration_ms));
            Ok((
                byte_length,
                sample_count,
                hash_file(&decoded, &cancellation, deadline)?,
            ))
        }
    })
    .await?;
    let models = preparation::prepare_models_for_job(
        job.app,
        job.model.preparation(),
        job.requester.attempt_id,
        "Preparing source separation",
        job.cancellation,
        &mut |_| {},
    )
    .await?;
    let runtime = crate::foundation::blocking::run({
        let app = job.app.clone();
        move || runtimes::inference::resolve(&app)
    })
    .await?;
    let device_profile =
        device_profiles::separation::profile(&runtime, job.model, job.cancellation).await?;
    let separated = separation::separate_to_paths(
        job.app,
        &models,
        device_profile,
        SpeechSeparateParams {
            identity: InferenceRequestIdentity {
                request_id: job.requester.request_id.into(),
                owner_kind: job.requester.owner_kind.into(),
                owner_id: job.requester.owner_id.into(),
                attempt_id: job.requester.attempt_id.into(),
                attempt_epoch: job.requester.attempt_epoch,
                input_revision: job.source.source_revision.clone(),
                model_spec_id: job.model.separation_spec()?.into(),
                budget_ms: budget_ms(job.source.duration_ms),
            },
            audio_ref: AudioFileReference {
                path: decoded.to_string_lossy().into_owned(),
                media_type: "audio/wav".into(),
                byte_length,
                sha256,
                sample_rate: SAMPLE_RATE,
                channels: 2,
                sample_format: "pcm_s16le".into(),
            },
        },
        job.cancellation,
        Priority::Background,
        job.targets,
    )
    .await?;
    std::fs::remove_file(&decoded).map_err(|error| error.to_string())?;
    if separated.background.sample_count != sample_count {
        let _ = std::fs::remove_file(job.targets.vocals);
        let _ = std::fs::remove_file(job.targets.background);
        return Err("invalid_output: separated background timeline differs from its source".into());
    }
    Ok(separated)
}
