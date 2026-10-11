use std::{path::Path, time::Instant};

use serde::Serialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use crate::{
    foundation::blocking,
    local_resources::preparation,
    media::transcript::{MediaTranscript, TranscriptSegment},
    spaces::assets::{authorize_spaces_asset, media_analysis},
};
use pipeline::TranscriptionPipeline;

pub(crate) mod alignment;
pub(crate) mod diarization;

pub(crate) fn engine_id() -> Result<&'static str, String> {
    crate::ai::local_runtime::speech::transcription_profile(
        crate::local_resources::model_catalog::LocalModelId::WhisperSmall,
    )
    .map(|profile| match profile.engine {
        crate::ai::contracts::speech_wire::AsrEngine::WhisperCpp => "whisper-cpp",
    })
}

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
    pub(in crate::spaces) fn progress(&self) -> u8 {
        self.progress
    }

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
    let (cancellation, _lease) = registry::acquire(job_id.clone())?;
    transcribe_media(
        app,
        job_id,
        source_path,
        source_revision,
        on_progress,
        cancellation,
    )
    .await
}

pub(in crate::spaces) async fn transcribe_media(
    app: tauri::AppHandle,
    job_id: String,
    source_path: String,
    source_revision: String,
    on_progress: Channel<MediaTranscriptionProgress>,
    cancellation: CancellationToken,
) -> Result<MediaTranscript, String> {
    crate::ai::local_runtime::speech::transcription_profile(
        crate::local_resources::model_catalog::LocalModelId::WhisperSmall,
    )?;
    let source = authorize_spaces_asset(&app, Path::new(&source_path))?;
    let inspected = source.clone();
    let revision = source_revision;
    let expected = revision.clone();
    blocking::run(move || crate::file_inspection::verify_content_revision(&inspected, &expected))
        .await?;
    let model = crate::local_resources::model_selection::transcription(&app)?;
    let prepared = preparation::prepare_models_for_job(
        &app,
        preparation::contracts::ResourcePreparationTarget::Model { model },
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
            format!("local-resource-unavailable: {error}")
        }
    })?;
    let source = authorize_spaces_asset(&app, &source)?;
    transcribe(
        &app,
        &job_id,
        &source,
        &revision,
        &on_progress,
        &cancellation,
        &prepared,
    )
    .await
}

async fn transcribe(
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
    let preparation_app = app.clone();
    let preparation_source = source.to_path_buf();
    let preparation_revision = revision.to_owned();
    let preparation_cancellation = cancellation.clone();
    let audio = blocking::run(move || {
        media_analysis::prepare(
            &preparation_app,
            &preparation_source,
            &preparation_revision,
            &preparation_cancellation,
        )
    })
    .await?;
    let pipeline = TranscriptionPipeline {
        app,
        job_id,
        source_revision: revision,
        progress,
        cancellation,
        resources,
    };
    let transcribed = pipeline.run(&audio).await;
    match transcribed {
        Ok(transcript) => {
            if cancellation.is_cancelled() {
                return Err(crate::media::TRANSCRIPTION_CANCELLED.into());
            }
            let verified_source = source.to_path_buf();
            let verified_revision = revision.to_owned();
            blocking::run(move || {
                crate::file_inspection::verify_content_revision(
                    &verified_source,
                    &verified_revision,
                )
            })
            .await?;
            send_progress(progress, "done", 100);
            log::info!(
                "media_transcription job={job_id} event=completed elapsed_ms={}",
                started.elapsed().as_millis()
            );
            Ok(transcript)
        }
        Err(error) => {
            log::error!(
                "media_transcription job={job_id} event=failed elapsed_ms={} error={error}",
                started.elapsed().as_millis()
            );
            Err(error)
        }
    }
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
