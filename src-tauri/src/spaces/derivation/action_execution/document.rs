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
    let document = request
        .pointer("/document/data")
        .and_then(Value::as_object)
        .ok_or("action-input-invalid: presentation document data is required")?;
    let node_id = request
        .pointer("/document/nodeId")
        .and_then(Value::as_str)
        .ok_or("action-input-invalid: source document is required")?;
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = pool.begin().await.map_err(crate::spaces::db::sql_error)?;
    let citations = super::super::document_citations::load(&mut tx, node_id).await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    let source = crate::documents::generation::document::DocumentSource {
        title: required_text(document, "title")?.to_owned(),
        markdown: required_text(document, "text")?.to_owned(),
        citations,
    };
    let format = super::super::document_formats::require(action_id)?;
    if let Err(error) = super::super::document_publication::start(
        app,
        request.clone(),
        action_id,
        format.engine,
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
        let bytes =
            crate::documents::generation::worker::render(format.kind, source, cancellation).await?;
        if cancellation.is_cancelled() {
            return Err(AppError::cancelled("Document rendering was cancelled"));
        }
        Ok(serialize(
            super::super::document_publication::publish(
                app,
                request,
                bytes,
                action_id,
                format.engine,
                authority,
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
