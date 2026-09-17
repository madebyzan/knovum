use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::access::authorize_spaces_asset;
use crate::spaces::knowledge::indexing::revision as knowledge_revision;
use node_payload::{load_node_data, required_string};
use persistence::persist_processing_state;

pub const ASSET_PROCESSING_CANCELLED: &str = "SPACE_ASSET_PROCESSING_CANCELLED";

pub async fn enrich_node_with_cancellation(
    app: &AppHandle,
    space_id: String,
    node_id: String,
    cancellation: &CancellationToken,
    updates: Option<&tauri::ipc::Channel<Value>>,
    job_id: Option<&str>,
) -> Result<Value, String> {
    let mut data = load_node_data(app, &space_id, &node_id).await?;
    let source = required_string(&data, "assetPath")?;
    let file_name = required_string(&data, "fileName")?;
    let revision = required_string(&data, "sourceRevision")?;
    let guard = persistence::PublicationGuard {
        source_revision: Some(&revision),
        job_id,
    };
    let canonical = authorize_spaces_asset(app, &PathBuf::from(source))?;
    let kind = required_string(&data, "kind")?;
    ensure_active(cancellation)?;
    let app_handle = app.clone();
    let preview_revision = revision.clone();
    let current = data.clone();
    let preview_cancellation = cancellation.clone();
    let preview = match crate::blocking::run(move || {
        pipeline::prepare_visual(
            &app_handle,
            &canonical,
            &file_name,
            &preview_revision,
            &kind,
            &current,
            &preview_cancellation,
        )
    })
    .await
    {
        Ok(value) => value,
        Err(error) => {
            return finish_enrichment_error(app, &space_id, &node_id, &mut data, error, &guard)
                .await;
        }
    };
    data = preview.data;
    persist_processing_state(
        app,
        &space_id,
        &node_id,
        &mut data,
        "processing",
        None,
        Some(&guard),
    )
    .await?;
    if let Some(channel) = updates {
        if let Err(error) = channel.send(data.clone()) {
            log::debug!("asset preview subscriber disconnected: {error}");
        }
    }
    let canonical =
        authorize_spaces_asset(app, &PathBuf::from(required_string(&data, "assetPath")?))?;
    let cancellation = cancellation.clone();
    let app_handle = app.clone();
    let preview_asset = preview.asset;
    let preview_data = data.clone();
    let content_revision = revision.clone();
    let enrichment = match crate::blocking::run(move || {
        pipeline::enrich_content(
            &app_handle,
            &canonical,
            &content_revision,
            preview_asset,
            preview_data,
            &cancellation,
        )
    })
    .await
    {
        Ok(value) => value,
        Err(error) => {
            return finish_enrichment_error(app, &space_id, &node_id, &mut data, error, &guard)
                .await;
        }
    };
    data = enrichment.data;
    knowledge_revision::apply(&mut data)?;
    persist_processing_state(
        app,
        &space_id,
        &node_id,
        &mut data,
        "done",
        Some(enrichment.chunks),
        Some(&guard),
    )
    .await?;
    Ok(data)
}

async fn finish_enrichment_error(
    app: &AppHandle,
    space_id: &str,
    node_id: &str,
    data: &mut Value,
    error: String,
    guard: &persistence::PublicationGuard<'_>,
) -> Result<Value, String> {
    let state = if error == ASSET_PROCESSING_CANCELLED {
        "done"
    } else {
        "failed"
    };
    persist_processing_state(app, space_id, node_id, data, state, None, Some(guard)).await?;
    Err(error)
}

pub(in crate::spaces) mod commit;
pub(in crate::spaces::assets) mod content;
pub(crate) mod document;
mod fields;
pub(crate) mod jobs;
mod node_payload;
pub(super) mod persistence;
mod pipeline;
pub(super) mod refresh;
pub(super) mod registry;
mod text_chunks;
pub(crate) use fields::apply_asset_fields;

pub(crate) fn ensure_active(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err(ASSET_PROCESSING_CANCELLED.to_owned())
    } else {
        Ok(())
    }
}
