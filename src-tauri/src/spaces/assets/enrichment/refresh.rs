use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use super::super::{files, PreparedAsset};
use super::{
    apply_asset_fields,
    node_payload::{load_node_content, optional_string, required_string, set_optional},
    persistence::{persist_processing_state, take_chunks},
};
use crate::app_error::AppError;
use crate::spaces::knowledge::model::revision;

pub(in crate::spaces::assets) async fn refresh_node(
    app: &AppHandle,
    space_id: String,
    node_id: String,
) -> Result<Option<Value>, AppError> {
    let data = load_node_content(app, &space_id, &node_id).await?;
    if let Some(state) = data.get("processingState").and_then(Value::as_str) {
        if state == "importing" || state == "processing" {
            return Ok(None);
        }
    }
    let old_previews = data
        .get("previewPath")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .into_iter()
        .collect();
    let refreshed_content = refresh_content(app, &space_id, &node_id, &data).await;
    // Only this node's former previews are candidates; never scan another in-flight generation.
    if refreshed_content.is_ok() {
        if let Err(error) = crate::spaces::previews::reclaim_unreferenced(app, old_previews).await {
            log::warn!("resource preview reclamation failed for {node_id}: {error}");
        }
    }
    refreshed_content
}

async fn refresh_content(
    app: &AppHandle,
    space_id: &str,
    node_id: &str,
    payload: &Value,
) -> Result<Option<Value>, AppError> {
    let source = required_string(payload, "assetPath")?;
    let file_name = required_string(payload, "fileName")?;
    let known_revision = optional_string(payload, "sourceRevision");
    let known_size = payload.get("size").and_then(Value::as_u64);
    let known_modified_at = payload.get("modifiedAt").and_then(Value::as_u64);
    let app_handle = app.clone();
    let refreshed = crate::blocking::run(move || {
        files::refresh_asset(
            &app_handle,
            files::RefreshAssetRequest {
                source: PathBuf::from(source),
                file_name,
                known_revision,
                known_size,
                known_modified_at,
            },
        )
    })
    .await?;
    let Some(mut asset) = refreshed.asset else {
        let mut current = load_node_content(app, space_id, node_id).await?;
        if apply_metadata_fields(&mut current, refreshed.size, refreshed.modified_at)? {
            persist_processing_state(app, space_id, node_id, &mut current, "done", None, None)
                .await?;
            return Ok(Some(current));
        }
        return Ok(None);
    };
    let created = asset.preview_path.clone().into_iter().collect();
    let refreshed_content = publish_refresh(app, space_id, node_id, payload, &mut asset).await;
    if refreshed_content.is_err() {
        if let Err(error) = crate::spaces::previews::reclaim_unreferenced(app, created).await {
            log::warn!("unpublished resource preview reclamation failed for {node_id}: {error}");
        }
    }
    refreshed_content
}

async fn publish_refresh(
    app: &AppHandle,
    space_id: &str,
    node_id: &str,
    payload: &Value,
    asset: &mut PreparedAsset,
) -> Result<Option<Value>, AppError> {
    let mut current = load_node_content(app, space_id, node_id).await?;
    apply_refresh_fields(&mut current, asset)?;
    revision::apply(&mut current)?;
    let chunks = take_chunks(asset);
    let guard = super::persistence::PublicationGuard {
        source_revision: payload.get("sourceRevision").and_then(Value::as_str),
        job_id: None,
    };
    persist_processing_state(
        app,
        space_id,
        node_id,
        &mut current,
        "done",
        Some(chunks),
        Some(&guard),
    )
    .await?;
    Ok(Some(current))
}

fn apply_metadata_fields(
    payload: &mut Value,
    size: Option<u64>,
    modified_at: Option<u64>,
) -> Result<bool, AppError> {
    let object = payload
        .as_object_mut()
        .ok_or_else(|| "Space node data must be an object".to_owned())?;
    let mut changed = false;
    if let Some(size) = size {
        changed |= object.get("size").and_then(Value::as_u64) != Some(size);
        object.insert("size".into(), Value::Number(size.into()));
    }
    if let Some(modified_at) = modified_at {
        changed |= object.get("modifiedAt").and_then(Value::as_u64) != Some(modified_at);
        object.insert("modifiedAt".into(), Value::Number(modified_at.into()));
    }
    Ok(changed)
}

fn apply_refresh_fields(payload: &mut Value, asset: &PreparedAsset) -> Result<(), AppError> {
    apply_asset_fields(payload, asset)?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| "Space node data must be an object".to_owned())?;
    set_optional(object, "sourceRevision", asset.source_revision.as_ref());
    set_optional(object, "previewLanguage", asset.preview_language.as_ref());
    if let Some(size) = asset.size {
        object.insert("size".into(), Value::Number(size.into()));
    }
    if let Some(modified_at) = asset.modified_at {
        object.insert("modifiedAt".into(), Value::Number(modified_at.into()));
    }
    Ok(())
}
