mod persistence;
mod registry;
mod render;
pub(super) mod srt;
mod start;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use super::{failure, CompletedDerivationOutput};
use crate::blocking;

pub const CANCELLED: &str = "VIDEO_SUBTITLE_DERIVATION_CANCELLED";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSubtitledVideoRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub output_node_id: String,
    pub source_revision: String,
    pub subtitle_node_id: String,
    pub subtitle_revision: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitledVideoProgress {
    stage: &'static str,
    progress: u8,
}

pub(super) async fn create(
    app: AppHandle,
    request: CreateSubtitledVideoRequest,
    on_progress: Channel<SubtitledVideoProgress>,
) -> Result<CompletedDerivationOutput, String> {
    let (cancellation, _lease) = registry::acquire(request.id.clone())?;
    let source = start::start(&app, &request).await?;
    let execution_app = app.clone();
    let execution_id = request.id.clone();
    let rendered = blocking::run(move || {
        render::render(
            &execution_app,
            &execution_id,
            source,
            &on_progress,
            &cancellation,
        )
    })
    .await;
    match rendered {
        Ok(output) => {
            let path = output.path.clone();
            match persistence::complete(&app, &request.id, output).await {
                Ok(completed) => Ok(completed),
                Err(error) => {
                    tokio::fs::remove_file(path).await.map_err(|cleanup| {
                        format!("{error}; derived video cleanup failed: {cleanup}")
                    })?;
                    Err(error)
                }
            }
        }
        Err(error) => {
            failure::fail_message(&app, &request.id, &error).await?;
            Err(error)
        }
    }
}

pub(super) async fn cancel(id: String) -> Result<(), String> {
    registry::cancel(&id)
}

pub(super) fn progress(
    channel: &Channel<SubtitledVideoProgress>,
    stage: &'static str,
    value: u8,
) -> Result<(), String> {
    channel
        .send(SubtitledVideoProgress {
            stage,
            progress: value,
        })
        .map_err(|error| error.to_string())
}

pub(super) fn subtitle_text(value: &Value) -> Result<String, String> {
    srt::from_subtitle(value)
}
