use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use super::super::{files, PreparedAsset};
use super::{
    apply_asset_fields,
    node_payload::{load_node_data, optional_string, required_string, set_optional},
    persistence::{persist_processing_state, take_chunks},
};
use crate::spaces::knowledge::indexing::revision;

pub(in crate::spaces::assets) async fn refresh_node(
    app: &AppHandle,
    space_id: String,
    node_id: String,
) -> Result<Option<Value>, String> {
    let data = load_node_data(app, &space_id, &node_id).await?;
    if let Some(state) = data.get("processingState").and_then(Value::as_str) {
        if state == "importing" || state == "processing" {
            return Ok(None);
        }
    }
    let source = required_string(&data, "assetPath")?;
    let file_name = required_string(&data, "fileName")?;
    let known_revision = optional_string(&data, "sourceRevision");
    let known_size = data.get("size").and_then(Value::as_u64);
    let known_modified_at = data.get("modifiedAt").and_then(Value::as_u64);
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
        let mut current = load_node_data(app, &space_id, &node_id).await?;
        if apply_metadata_fields(&mut current, refreshed.size, refreshed.modified_at)? {
            persist_processing_state(app, &space_id, &node_id, &mut current, "done", None, None)
                .await?;
            return Ok(Some(current));
        }
        return Ok(None);
    };
    let mut current = load_node_data(app, &space_id, &node_id).await?;
    apply_refresh_fields(&mut current, &asset)?;
    revision::apply(&mut current)?;
    let chunks = take_chunks(&mut asset);
    let guard = super::persistence::PublicationGuard {
        source_revision: data.get("sourceRevision").and_then(Value::as_str),
        job_id: None,
    };
    persist_processing_state(
        app,
        &space_id,
        &node_id,
        &mut current,
        "done",
        Some(chunks),
        Some(&guard),
    )
    .await?;
    Ok(Some(current))
}

fn apply_metadata_fields(
    data: &mut Value,
    size: Option<u64>,
    modified_at: Option<u64>,
) -> Result<bool, String> {
    let object = data
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

fn apply_refresh_fields(data: &mut Value, asset: &PreparedAsset) -> Result<(), String> {
    apply_asset_fields(data, asset)?;
    let object = data
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
