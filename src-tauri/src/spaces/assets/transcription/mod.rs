use std::{path::Path, time::Instant};

use serde::Serialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use crate::{
    blocking,
    local_resources::{models::catalog, preparation, tools},
    media::{
        compute_budget::ComputeBudget,
        transcript::{MediaTranscript, TranscriptSegment},
        workspace::AnalysisWorkspace,
    },
    spaces::assets::{authorize_spaces_asset, media_analysis},
};
use pipeline::TranscriptionPipeline;

#[derive(Clone, Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct MediaTranscriptionProgress {
    stage: String,
    progress: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    partial: Option<MediaTranscriptionPartial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<preparation::contracts::ResourcePreparationProgress>,
}

impl MediaTranscriptionProgress {
    pub(crate) fn stage(stage: &str, progress: u8) -> Self {
        Self {
            stage: stage.into(),
            progress,
            partial: None,
            resources: None,
        }
    }
}

#[derive(Clone, Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub(crate) struct MediaTranscriptionPartial {
    language: String,
    segments: Vec<TranscriptSegment>,
}

pub(in crate::spaces::assets) async fn start_transcription(
    app: tauri::AppHandle,
    job_id: String,
    source_path: String,
    source_revision: String,
    on_progress: Channel<MediaTranscriptionProgress>,
) -> Result<MediaTranscript, String> {
    let source = authorize_spaces_asset(&app, Path::new(&source_path))?;
    let (cancellation, _lease) = registry::acquire(job_id.clone())?;
    let inspected = source.clone();
    let revision = source_revision;
    let expected = revision.clone();
    blocking::run(move || crate::file_inspection::verify_content_revision(&inspected, &expected))
        .await?;
    let prepared = preparation::prepare_models_for_job(
        &app,
        preparation::contracts::ResourcePreparationTarget::Transcription,
        &format!("resource:{job_id}"),
        "Preparing speech recognition",
        &cancellation,
        &mut |resources| {
            observe(
                &on_progress,
                MediaTranscriptionProgress {
                    stage: "preparingResources".into(),
                    progress: 0,
                    partial: None,
                    resources: Some(resources),
                },
            );
        },
    )
    .await
    .map_err(|error| {
        if cancellation.is_cancelled() || error == preparation::RESOURCE_PREPARATION_CANCELLED {
            crate::media::TRANSCRIPTION_CANCELLED.into()
        } else {
            error
        }
    })?;
    let execution = registry::execution_permit(&cancellation).await?;
    let source = authorize_spaces_asset(&app, &source)?;
    blocking::run(move || {
        let _execution = execution;
        let _job = _lease;
        crate::file_inspection::verify_content_revision(&source, &revision)?;
        transcribe(
            &app,
            &job_id,
            &source,
            &revision,
            &on_progress,
            &cancellation,
            &prepared,
        )
    })
    .await
}

fn transcribe(
    app: &tauri::AppHandle,
    job_id: &str,
    source: &Path,
    revision: &str,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
    resources: &preparation::PreparedModels,
) -> Result<MediaTranscript, String> {
    if cancellation.is_cancelled() {
        return Err(crate::media::TRANSCRIPTION_CANCELLED.into());
    }
    let started = Instant::now();
    log::info!("media_transcription job={job_id} event=started");
    send_progress(progress, "preparing", 3);
    let audio = media_analysis::prepare(app, source, revision, cancellation)?;
    let whisper = required_tool(
        app,
        "KNOVUM_WHISPER_PATH",
        tools::whisper_cli_name(),
        "whisper.cpp CLI",
    )?;
    let model = resources.path(catalog::ModelKind::Transcription)?;
    let vad_model = resources.path(catalog::ModelKind::TranscriptionVad)?;
    let compute_budget = ComputeBudget::detect()?;
    let paths = AnalysisWorkspace::new(app)?;
    let pipeline = TranscriptionPipeline {
        job_id,
        progress,
        cancellation,
        whisper_cli: &whisper,
        model,
        vad_model,
        paths: &paths,
        compute_budget,
    };
    let transcribed = pipeline.run(&audio).and_then(|transcript| {
        if cancellation.is_cancelled() {
            return Err(crate::media::TRANSCRIPTION_CANCELLED.into());
        }
        crate::file_inspection::verify_content_revision(source, revision)?;
        Ok(transcript)
    });
    let cleanup = paths.cleanup();
    match (transcribed, cleanup) {
        (Ok(transcript), Ok(())) => {
            send_progress(progress, "done", 100);
            log::info!(
                "media_transcription job={job_id} event=completed elapsed_ms={}",
                started.elapsed().as_millis()
            );
            Ok(transcript)
        }
        (Err(error), cleanup) => {
            if let Err(cleanup_error) = cleanup {
                log::warn!("media_transcription job={job_id} cleanup failed: {cleanup_error}");
            }
            log::error!(
                "media_transcription job={job_id} event=failed elapsed_ms={} error={error}",
                started.elapsed().as_millis()
            );
            Err(error)
        }
        (Ok(_), Err(error)) => Err(error),
    }
}

fn required_tool(
    app: &tauri::AppHandle,
    env_key: &str,
    name: &str,
    label: &str,
) -> Result<std::path::PathBuf, String> {
    tools::resolve_tool(app, env_key, name)?
        .ok_or_else(|| format!("{label} is required and is not installed"))
}

pub(crate) fn send_progress(
    channel: &Channel<MediaTranscriptionProgress>,
    stage: &str,
    progress: u8,
) {
    observe(channel, MediaTranscriptionProgress::stage(stage, progress));
}

pub(crate) fn send_partial_progress(
    channel: &Channel<MediaTranscriptionProgress>,
    progress: u8,
    language: &str,
    segments: Vec<TranscriptSegment>,
) {
    if segments.is_empty() {
        return send_progress(channel, "transcribing", progress);
    }
    observe(
        channel,
        MediaTranscriptionProgress {
            stage: "transcribing".to_owned(),
            progress,
            resources: None,
            partial: Some(MediaTranscriptionPartial {
                language: language.to_owned(),
                segments,
            }),
        },
    );
}

fn observe(channel: &Channel<MediaTranscriptionProgress>, progress: MediaTranscriptionProgress) {
    if let Err(error) = channel.send(progress) {
        log::debug!("Transcription progress observer disconnected: {error}");
    }
}

mod pipeline;
mod registry;
pub(in crate::spaces::assets) use registry::cancel as cancel_transcription;
