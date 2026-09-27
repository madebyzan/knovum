use std::path::Path;

use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

use crate::spaces::assets::{
    media_analysis,
    transcription::{self, MediaTranscriptionProgress},
};

use super::{node::required_string, RunEnrichmentJobRequest};

pub(super) async fn analyze(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    data: &mut Value,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let source = media_source(data)?;
    let app = app.clone();
    let revision = request.source_revision.clone();
    let token = cancellation.clone();
    transcription::send_progress(progress, "analyzing", 5);
    let prepared = crate::blocking::run(move || {
        media_analysis::prepare(&app, Path::new(&source), &revision, &token)
    })
    .await?;
    data.as_object_mut()
        .ok_or("Space node data must be an object")?
        .insert(
            "mediaAnalysis".into(),
            serde_json::json!({
                "schemaVersion": 1,
                "sourceRevision": request.source_revision,
                "durationMs": prepared.analysis.duration_ms,
                "waveform": prepared.analysis.waveform,
            }),
        );
    transcription::send_progress(progress, "analyzing", 95);
    Ok(())
}

pub(super) async fn transcribe(
    app: &AppHandle,
    job_id: &str,
    request: &RunEnrichmentJobRequest,
    data: &mut Value,
    progress: Channel<MediaTranscriptionProgress>,
) -> Result<(), String> {
    let source = media_source(data)?;
    if data["mediaAnalysis"]["sourceRevision"].as_str() != Some(request.source_revision.as_str()) {
        return Err("Transcription requires current media analysis".into());
    }
    let transcript = transcription::start_transcription(
        app.clone(),
        job_id.to_owned(),
        source.clone(),
        request.source_revision.clone(),
        progress,
    )
    .await?;
    let revision = request.source_revision.clone();
    crate::blocking::run(move || {
        crate::file_inspection::verify_content_revision(Path::new(&source), &revision)
    })
    .await?;
    data.as_object_mut()
        .ok_or("Space node data must be an object")?
        .insert(
            "mediaKnowledge".into(),
            serde_json::json!({
                "transcript": transcript,
                "sourceRevision": request.source_revision,
                "schemaVersion": 4,
            }),
        );
    Ok(())
}

fn media_source(data: &Value) -> Result<String, String> {
    if !matches!(
        data.get("kind").and_then(Value::as_str),
        Some("audio" | "video")
    ) {
        return Err("Media processing requires an audio or video node".into());
    }
    Ok(required_string(data, "assetPath")?.to_owned())
}
