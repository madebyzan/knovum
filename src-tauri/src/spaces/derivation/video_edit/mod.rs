mod ass;
mod contracts;
mod filter_graph;
pub(crate) mod fonts;
mod persistence;
mod proxy;
mod proxy_registry;
mod registry;
mod render;
mod start;
mod validation;

#[cfg(test)]
mod tests;

use tauri::{ipc::Channel, AppHandle};

use super::{failure, CompletedDerivationOutput};
use crate::{foundation::app_error::AppError, foundation::blocking};

pub use contracts::{
    PrepareVideoEditProxyRequest, RenderVideoEditRequest, VideoEditProgress, VideoEditProject,
};
use contracts::{TextPlacement, TextPreset};
pub(in crate::spaces) use validation::validate_project;

pub const CANCELLED: &str = "video-edit-render-cancelled";

pub async fn render(
    app: AppHandle,
    request: RenderVideoEditRequest,
    progress: Channel<VideoEditProgress>,
) -> Result<CompletedDerivationOutput, AppError> {
    let (cancellation, _lease) = registry::acquire(request.id.clone())?;
    let source = start::start(&app, &request).await?;
    let execution_app = app.clone();
    let rendered =
        blocking::run(move || render::render(&execution_app, source, &progress, &cancellation))
            .await
            .map_err(|message| {
                if message == CANCELLED {
                    AppError::new("video_edit_render_cancelled", "Video render cancelled")
                } else {
                    AppError::from(message)
                }
            });
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
            failure::fail_message(&app, &request.id, &error.message).await?;
            Err(error)
        }
    }
}

pub fn cancel(derivation_id: &str) -> Result<(), String> {
    registry::cancel(derivation_id)
}

pub async fn prepare_proxy(
    app: AppHandle,
    request: PrepareVideoEditProxyRequest,
) -> Result<String, String> {
    proxy::prepare(app, request).await
}

pub fn cancel_proxy(job_id: &str) -> Result<(), String> {
    proxy_registry::cancel(job_id)
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
