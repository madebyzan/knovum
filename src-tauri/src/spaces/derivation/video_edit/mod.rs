mod ass;
mod filter_graph;
pub(crate) mod fonts;
mod persistence;
mod proxy;
mod proxy_registry;
mod registry;
mod render;
mod start;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, AppHandle};

use super::{failure, CompletedDerivationOutput};
use crate::blocking;

pub use proxy::PrepareRequest;

pub const CANCELLED: &str = "VIDEO_EDIT_RENDER_CANCELLED";

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoEditProject {
    pub version: u8,
    pub source_revision: String,
    pub source_width: u32,
    pub source_height: u32,
    pub video_track: VideoTrack,
    pub text_track: TextTrack,
    pub transform: VideoTransform,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTrack {
    pub clips: Vec<VideoClip>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoClip {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub volume: f64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextTrack {
    pub items: Vec<TextOverlay>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextOverlay {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub placement: TextPlacement,
    pub style: TextStyle,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextStyle {
    pub family: String,
    pub weight: u16,
    pub size: u16,
    pub color: String,
    pub preset: TextPreset,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextPreset {
    Plain,
    Outline,
    Shadow,
    Badge,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTransform {
    pub rotation: u16,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextPlacement {
    Top,
    Center,
    Bottom,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderVideoEditRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub output_node_id: String,
    pub source_revision: String,
    pub project: VideoEditProject,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEditProgress {
    stage: &'static str,
    progress: u8,
}

pub async fn create(
    app: AppHandle,
    request: RenderVideoEditRequest,
    progress: Channel<VideoEditProgress>,
) -> Result<CompletedDerivationOutput, String> {
    let (cancellation, _lease) = registry::acquire(request.id.clone())?;
    let source = start::start(&app, &request).await?;
    let execution_app = app.clone();
    let rendered =
        blocking::run(move || render::render(&execution_app, source, &progress, &cancellation))
            .await;
    match rendered {
        Ok(output) => {
            let path = output.path.clone();
            match persistence::complete(&app, &request.id, output).await {
                Ok(completed) => Ok(completed),
                Err(error) => {
                    tokio::fs::remove_file(path).await.map_err(|cleanup| {
                        format!("{error}; rendered video cleanup failed: {cleanup}")
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

pub fn cancel(id: &str) -> Result<(), String> {
    registry::cancel(id)
}

pub async fn prepare_proxy(
    app: AppHandle,
    request: proxy::PrepareRequest,
) -> Result<String, String> {
    proxy::prepare(app, request).await
}

pub fn cancel_proxy(id: &str) -> Result<(), String> {
    proxy_registry::cancel(id)
}

fn report(
    channel: &Channel<VideoEditProgress>,
    stage: &'static str,
    progress: u8,
) -> Result<(), String> {
    channel
        .send(VideoEditProgress { stage, progress })
        .map_err(|error| error.to_string())
}
