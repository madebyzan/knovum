mod ass;
mod clip_filter;
mod contracts;
mod filter_graph;
pub(crate) mod fonts;
mod persistence;
mod proxy;
mod proxy_registry;
mod registry;
mod render;
mod render_inputs;
mod source_audio;
mod sources;
mod start;
mod timing;
mod validation;

#[cfg(test)]
mod tests;

use tauri::{ipc::Channel, AppHandle};

use super::{failure, CompletedDerivationOutput};
use crate::{foundation::app_error::AppError, foundation::blocking};

pub use contracts::{
    PrepareVideoEditProxyRequest, RenderVideoEditRequest, VideoEditProgress, VideoEditProject,
};
pub(in crate::spaces) use validation::validate_project;

pub const CANCELLED: &str = "video-edit-render-cancelled";

pub async fn render(
    app: AppHandle,
    request: RenderVideoEditRequest,
    progress: Channel<VideoEditProgress>,
) -> Result<CompletedDerivationOutput, AppError> {
    let (cancellation, _lease) = registry::acquire(request.id.clone())?;
    let source = match start::start(&app, &request).await {
        Ok(source) => source,
        Err(message) => {
            let settlement = failure::fail_pending(
                &app,
                &request.space_id,
                &request.output_node_id,
                failure::PendingNodeState::Processing,
            )
            .await;
            return Err(failure::with_settlement(message.into(), settlement));
        }
    };
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
                    let settlement = failure::fail_message(&app, &request.id, &error.message).await;
                    let mut error = failure::with_settlement(error, settlement);
                    if let Err(cleanup) = tokio::fs::remove_file(path).await {
                        error
                            .message
                            .push_str(&format!("; rendered video cleanup failed: {cleanup}"));
                    }
                    Err(error)
                }
            }
        }
        Err(error) => {
            let settlement = failure::fail_message(&app, &request.id, &error.message).await;
            Err(failure::with_settlement(error, settlement))
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
