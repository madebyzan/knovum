use std::{path::Path, time::Instant};

use serde::Serialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use crate::{
    blocking,
    local_resources::{models, models::catalog, tools},
    media::{
        compute_budget::ComputeBudget,
        transcript::{MediaTranscript, TranscriptSegment},
        workspace::AnalysisWorkspace,
    },
    spaces::assets::authorize_spaces_asset,
};
use pipeline::TranscriptionPipeline;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTranscriptionProgress {
    stage: String,
    progress: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    partial: Option<MediaTranscriptionPartial>,
}

impl MediaTranscriptionProgress {
    pub(crate) fn stage(stage: &str, progress: u8) -> Self {
        Self {
            stage: stage.into(),
            progress,
            partial: None,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MediaTranscriptionPartial {
    language: String,
    segments: Vec<TranscriptSegment>,
}

#[tauri::command]
pub async fn transcribe_media(
    app: tauri::AppHandle,
    job_id: String,
    source_path: String,
    on_progress: Channel<MediaTranscriptionProgress>,
) -> Result<MediaTranscript, String> {
    let source = authorize_spaces_asset(&app, Path::new(&source_path))?;
    let (cancellation, _lease) = registry::acquire(job_id.clone())?;
    let execution_id = job_id.clone();
    let transcript = blocking::run(move || {
        transcribe(&app, &execution_id, &source, &on_progress, &cancellation)
    })
    .await;
    transcript
}

#[tauri::command]
pub async fn cancel_media_transcription(job_id: String) -> Result<(), String> {
    registry::cancel(&job_id)
}

fn transcribe(
    app: &tauri::AppHandle,
    job_id: &str,
    source: &Path,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<MediaTranscript, String> {
    let started = Instant::now();
    log::info!("media_transcription job={job_id} event=started");
    send_progress(progress, "preparing", 3)?;
    let ffmpeg = required_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name(), "FFmpeg")?;
    let whisper = required_tool(
        app,
        "KNOVUM_WHISPER_PATH",
        tools::whisper_cli_name(),
        "whisper.cpp CLI",
    )?;
    let model = models::require_verified_path(app, catalog::TRANSCRIPTION)?;
    let vad_model = models::require_verified_path(app, catalog::TRANSCRIPTION_VAD)?;
    let paths = AnalysisWorkspace::new(app)?;
    let pipeline = TranscriptionPipeline {
        job_id,
        progress,
        cancellation,
        ffmpeg: &ffmpeg,
        whisper_cli: &whisper,
        model: &model,
        vad_model: &vad_model,
        paths: &paths,
        compute_budget: ComputeBudget::detect()?,
    };
    let transcribed = pipeline.run(source);
    let cleanup = paths.cleanup();
    match (transcribed, cleanup) {
        (Ok(transcript), Ok(())) => {
            send_progress(progress, "done", 100)?;
            log::info!(
                "media_transcription job={job_id} event=completed elapsed_ms={}",
                started.elapsed().as_millis()
            );
            Ok(transcript)
        }
        (Err(error), _) => {
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
    tools::resolve_tool(app, env_key, name)
        .ok_or_else(|| format!("{label} is required and is not installed"))
}

pub(crate) fn send_progress(
    channel: &Channel<MediaTranscriptionProgress>,
    stage: &str,
    progress: u8,
) -> Result<(), String> {
    channel
        .send(MediaTranscriptionProgress {
            stage: stage.to_owned(),
            progress,
            partial: None,
        })
        .map_err(|error| error.to_string())
}

pub(crate) fn send_partial_progress(
    channel: &Channel<MediaTranscriptionProgress>,
    progress: u8,
    language: &str,
    segments: Vec<TranscriptSegment>,
) -> Result<(), String> {
    if segments.is_empty() {
        return send_progress(channel, "transcribing", progress);
    }
    channel
        .send(MediaTranscriptionProgress {
            stage: "transcribing".to_owned(),
            progress,
            partial: Some(MediaTranscriptionPartial {
                language: language.to_owned(),
                segments,
            }),
        })
        .map_err(|error| error.to_string())
}

mod pipeline;
mod registry;
