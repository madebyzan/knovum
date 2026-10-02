use std::path::Path;

use tauri::{
    ipc::{Channel, InvokeResponseBody},
    AppHandle,
};
use tokio_util::sync::CancellationToken;

use super::{
    persistence::VideoSubtitleSource, progress, RenderSubtitledVideoRequest,
    SubtitledVideoProgress, CANCELLED,
};
use crate::spaces::assets::{
    authorize_spaces_asset,
    transcription::{self, MediaTranscriptionProgress},
};

pub(super) enum PreparedSubtitles {
    Ready(String),
    Recognize,
}

pub(super) struct SubtitleVideoSource {
    pub path: String,
    pub subtitles: PreparedSubtitles,
}

pub(super) async fn prepare(
    app: &AppHandle,
    request: &RenderSubtitledVideoRequest,
    source: SubtitleVideoSource,
    channel: &Channel<SubtitledVideoProgress>,
    cancellation: &CancellationToken,
) -> Result<VideoSubtitleSource, String> {
    if cancellation.is_cancelled() {
        return Err(CANCELLED.into());
    }
    let path = authorize_spaces_asset(app, Path::new(&source.path))?;
    let expected = request.source_revision.clone();
    crate::blocking::run(move || crate::file_inspection::verify_content_revision(&path, &expected))
        .await?;
    let srt = match source.subtitles {
        PreparedSubtitles::Ready(srt) => srt,
        PreparedSubtitles::Recognize => {
            let observer = channel.clone();
            let speech_progress = Channel::new(move |body| {
                let InvokeResponseBody::Json(json) = body else {
                    return Err(std::io::Error::other("Invalid transcription progress").into());
                };
                let message: MediaTranscriptionProgress = serde_json::from_str(&json)?;
                progress(
                    &observer,
                    "recognizingSubtitles",
                    5 + ((u16::from(message.progress()) * 55 / 100) as u8),
                )
                .map_err(|error| std::io::Error::other(error).into())
            });
            let transcript = transcription::transcribe_media(
                app.clone(),
                request.id.clone(),
                source.path.clone(),
                request.source_revision.clone(),
                speech_progress,
                cancellation.clone(),
            )
            .await
            .map_err(|error| {
                if cancellation.is_cancelled() {
                    CANCELLED.into()
                } else {
                    error
                }
            })?;
            super::srt::from_cues(&transcript.subtitle_cues, transcript.duration_ms)?
        }
    };
    if cancellation.is_cancelled() {
        return Err(CANCELLED.into());
    }
    Ok(VideoSubtitleSource {
        path: source.path,
        srt,
    })
}
