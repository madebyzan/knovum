mod persistence;
mod preparation;
mod render;
pub(super) mod srt;
mod start;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, AppHandle};

use super::{failure, CompletedDerivationOutput};
use crate::foundation::app_error::AppError;
use crate::foundation::blocking;

pub const CANCELLED: &str = "video-subtitle-derivation-cancelled";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderSubtitledVideoRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub output_node_id: String,
    pub source_revision: String,
    pub knowledge_revision: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitledVideoProgress {
    stage: &'static str,
    progress: u8,
}

pub(super) async fn render_for_action(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: RenderSubtitledVideoRequest,
    on_progress: Channel<SubtitledVideoProgress>,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<CompletedDerivationOutput, AppError> {
    authority
        .require_active(crate::spaces::db::pool(&app).await?)
        .await?;
    let source =
        match start::start(crate::spaces::db::pool(&app).await?, &request, &authority).await {
            Ok(source) => source,
            Err(error) => {
                let settlement = failure::fail_pending(
                    &app,
                    &request.space_id,
                    &request.output_node_id,
                    failure::PendingNodeState::Processing,
                )
                .await;
                return Err(failure::with_settlement(error.into(), settlement));
            }
        };
    let execution_app = app.clone();
    let execution_id = request.id.clone();
    let rendered = async {
        let source =
            preparation::prepare(&app, &request, source, &on_progress, cancellation).await?;
        let render_cancellation = cancellation.clone();
        let expected = request.source_revision.clone();
        blocking::run(move || {
            let source_path = source.path.clone();
            let output = render::render(
                &execution_app,
                &execution_id,
                source,
                &on_progress,
                &render_cancellation,
            )?;
            if let Err(error) = crate::file_inspection::verify_content_revision(
                std::path::Path::new(&source_path),
                &expected,
            ) {
                std::fs::remove_file(&output.path).map_err(|cleanup| {
                    format!("{error}; derived video cleanup failed: {cleanup}")
                })?;
                return Err(error);
            }
            Ok(output)
        })
        .await
    }
    .await;
    match rendered {
        Ok(output) => {
            let path = output.path.clone();
            let enrichment_target =
                crate::spaces::assets::enrichment::jobs::intent::selection(&app)?;
            match persistence::complete(
                crate::spaces::db::pool(&app).await?,
                &request.id,
                output,
                &authority,
                cancellation,
                enrichment_target.as_ref(),
            )
            .await
            {
                Ok(completed) => Ok(completed),
                Err(error) => {
                    let mut primary = error;
                    if let Err(cleanup) = super::video_output::discard(&app, &path).await {
                        primary.message = format!(
                            "{}; derived video cleanup failed: {cleanup}",
                            primary.message
                        );
                    }
                    let settlement =
                        failure::fail_message(&app, &request.id, &primary.message).await;
                    Err(failure::with_settlement(primary, settlement))
                }
            }
        }
        Err(error) => {
            let error = if error == CANCELLED {
                AppError::new("cancelled", "Video subtitle rendering cancelled")
            } else {
                AppError::from(error)
            };
            let settlement = failure::fail_message(&app, &request.id, &error.message).await;
            Err(failure::with_settlement(error, settlement))
        }
    }
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
