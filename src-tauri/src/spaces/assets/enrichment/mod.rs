use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::access::authorize_spaces_asset;
use crate::app_error::AppError;
use crate::spaces::knowledge::model::revision as knowledge_revision;
use node_payload::{load_node_content, required_string};
use persistence::persist_processing_state;

pub(crate) struct FinalizedImport {
    pub data: Value,
    pub source_chunks: Vec<crate::spaces::knowledge::indexing::KnowledgeChunkInput>,
}

pub(crate) fn reserve_import_preview(
    app: &AppHandle,
    asset: &super::PreparedAsset,
) -> Result<Option<PathBuf>, String> {
    if asset.preview_language.as_deref() == Some("html")
        || asset.kind == super::kind::AssetKind::Markdown
    {
        return Ok(None);
    }
    let Some(source) = asset.path.as_deref() else {
        return Ok(None);
    };
    crate::spaces::previews::reserved_thumbnail_path(app, std::path::Path::new(source))
}

pub(crate) fn finalize_import(
    app: &AppHandle,
    mut asset: super::PreparedAsset,
    reserved_preview: Option<&std::path::Path>,
    cancellation: &CancellationToken,
    claim_preview_path: &mut dyn FnMut(&std::path::Path) -> Result<(), String>,
    claim_embedded_path: &mut dyn FnMut(&std::path::Path) -> Result<(), String>,
) -> Result<FinalizedImport, String> {
    let Some(source) = asset.path.as_deref().map(PathBuf::from) else {
        let mut data = super::node_payload::from_import(asset)?;
        knowledge_revision::apply(&mut data)?;
        return Ok(FinalizedImport {
            data,
            source_chunks: Vec::new(),
        });
    };
    let revision = asset
        .source_revision
        .clone()
        .ok_or("Imported asset has no source revision")?;
    if asset.kind != super::kind::AssetKind::Markdown {
        crate::spaces::previews::enrich_visual_at(
            app,
            &source,
            &mut asset,
            cancellation,
            reserved_preview,
            claim_preview_path,
        )?;
    }
    content::extract_owned(
        app,
        &source,
        &revision,
        &mut asset,
        cancellation,
        claim_embedded_path,
    )?;
    asset.processing_state = None;
    let source_chunks = persistence::take_chunks(&mut asset);
    let mut data = super::node_payload::from_import(asset)?;
    knowledge_revision::apply(&mut data)?;
    Ok(FinalizedImport {
        data,
        source_chunks,
    })
}

pub const ASSET_PROCESSING_CANCELLED: &str = "space-asset-processing-cancelled";

pub async fn enrich_node_with_cancellation(
    app: &AppHandle,
    space_id: String,
    node_id: String,
    cancellation: &CancellationToken,
    updates: Option<&tauri::ipc::Channel<Value>>,
    job_id: Option<&str>,
) -> Result<Value, AppError> {
    let mut data = load_node_content(app, &space_id, &node_id).await?;
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
    let preview_payload = data.clone();
    let content_revision = revision.clone();
    let enrichment = match crate::blocking::run(move || {
        pipeline::enrich_content(
            &app_handle,
            &canonical,
            &content_revision,
            preview_asset,
            preview_payload,
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
) -> Result<Value, AppError> {
    let state = if error == ASSET_PROCESSING_CANCELLED {
        "done"
    } else {
        "failed"
    };
    persist_processing_state(app, space_id, node_id, data, state, None, Some(guard)).await?;
    Err(processing_error(error))
}

pub(in crate::spaces) mod commit;
pub(in crate::spaces::assets) mod content;
pub(crate) mod document;
mod fields;
pub(crate) mod jobs;
pub(crate) mod media_knowledge;
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

pub(crate) fn processing_error(message: String) -> AppError {
    if message == ASSET_PROCESSING_CANCELLED {
        AppError::new(
            "space_asset_processing_cancelled",
            "Asset processing cancelled",
        )
    } else {
        AppError::from(message)
    }
}

#[cfg(test)]
mod tests;
