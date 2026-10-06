use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

use serde_json::Value;
use tauri::{AppHandle, Runtime};

use super::ClipboardPayload;
use crate::spaces::structure::{validate_space_snapshot, StructureLayout, StructureNode};

pub(super) fn validate<R: Runtime>(
    app: &AppHandle<R>,
    payload: &ClipboardPayload,
) -> Result<(), String> {
    if payload.version != 1 || payload.structure.is_empty() || payload.structure.len() > 100_000 {
        return Err("Unsupported clipboard structure format".into());
    }
    validate_space_snapshot(&payload.structure, &payload.layouts)?;
    let expected: HashSet<&str> = payload
        .structure
        .iter()
        .filter_map(|structure_node| match structure_node {
            StructureNode::Content { node_id, .. } => Some(node_id.as_str()),
            _ => None,
        })
        .collect();
    let actual: HashSet<&str> = payload.nodes.iter().map(|node| node.id.as_str()).collect();
    if actual.len() != payload.nodes.len() || actual != expected {
        return Err("Clipboard content identity mismatch".into());
    }
    validate_assets(app, payload)?;
    validate_relations(payload, &actual)
}

fn validate_assets<R: Runtime>(
    app: &AppHandle<R>,
    payload: &ClipboardPayload,
) -> Result<(), String> {
    for node in &payload.nodes {
        node.data
            .as_object()
            .ok_or("Clipboard node data must be an object")?;
        let mut paths = BTreeSet::new();
        crate::spaces::assets::visit_asset_references(&node.data, &mut |kind, path| {
            if kind != crate::spaces::assets::AssetReference::Display {
                paths.insert(path.to_owned());
            }
        });
        for path in paths {
            crate::spaces::assets::authorize_spaces_asset(app, Path::new(&path))?;
        }
    }
    for layout in &payload.layouts {
        let appearance = match layout {
            StructureLayout::Content { appearance, .. }
            | StructureLayout::Section { appearance, .. } => appearance.as_ref(),
            StructureLayout::Group { .. } => None,
        };
        if let Some(path) = appearance
            .and_then(|value| value.get("background"))
            .and_then(Value::as_str)
        {
            crate::spaces::assets::authorize_spaces_asset(app, Path::new(path))?;
        }
    }
    Ok(())
}

fn validate_relations(payload: &ClipboardPayload, nodes: &HashSet<&str>) -> Result<(), String> {
    let mut ids = HashSet::new();
    for relation in &payload.relations {
        if relation.id.is_empty()
            || relation.id.encode_utf16().count() > 128
            || !ids.insert(relation.id.as_str())
            || relation.source_id == relation.target_id
            || !nodes.contains(relation.source_id.as_str())
            || !nodes.contains(relation.target_id.as_str())
        {
            return Err("Clipboard relation is invalid".into());
        }
        if relation.status_before_invalidation.is_some()
            || !matches!(relation.status.as_str(), "active" | "contested")
        {
            return Err("Clipboard relation lifecycle is invalid".into());
        }
        if !matches!(
            relation.actor_kind.as_str(),
            "human" | "builtin_agent" | "external_agent" | "system"
        ) {
            return Err("Clipboard relation actor is invalid".into());
        }
    }
    Ok(())
}

mod remap;
pub(super) use remap::{remap, RemappedPayload};
