use serde_json::Value;
use tauri::AppHandle;

use super::super::failure::with_settlement;
use super::{required_text, serialize, ExecutionAuthority};
use crate::app_error::AppError;

pub(super) async fn execute(
    app: &AppHandle,
    action_id: &str,
    request: Value,
    authority: &ExecutionAuthority,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Value, AppError> {
    authority
        .require_active(crate::spaces::db::pool(app).await?)
        .await?;
    let target = super::super::document_formats::require(action_id)?;
    let engine = target.engine_for_source(&request)?;
    if let Err(error) = super::super::document_publication::start(
        app,
        request.clone(),
        action_id,
        &engine,
        authority,
    )
    .await
    {
        let reset = super::super::failure::fail_pending(
            app,
            required_text(
                request.as_object().ok_or("document request is invalid")?,
                "spaceId",
            )?,
            required_text(
                request.as_object().ok_or("document request is invalid")?,
                "outputNodeId",
            )?,
            super::super::failure::PendingNodeState::Processing,
        )
        .await;
        return Err(with_settlement(error, reset));
    }
    let run_id = required_text(
        request.as_object().ok_or("document request is invalid")?,
        "id",
    )?
    .to_owned();
    let execution: Result<Value, AppError> = async {
        let source = super::super::document_source::load(app, &request).await?;
        let file = match &source {
            crate::documents::generation::input::DocumentInput::File(file) => Some(file.clone()),
            _ => None,
        };
        let bytes =
            crate::documents::generation::worker::render(target.format, source, cancellation)
                .await?;
        if let Some(file) = file {
            crate::blocking::run(move || file.verify_current()).await?;
        }
        if cancellation.is_cancelled() {
            return Err(AppError::cancelled("Document rendering was cancelled"));
        }
        Ok(serialize(
            super::super::document_publication::publish(
                app, request, bytes, action_id, &engine, authority,
            )
            .await?,
        )?)
    }
    .await;
    match execution {
        Ok(output) => Ok(output),
        Err(error) => {
            let settlement =
                super::super::failure::fail_message(app, &run_id, &error.message).await;
            Err(with_settlement(error, settlement))
        }
    }
}
