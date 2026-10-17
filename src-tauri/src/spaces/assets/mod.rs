mod access;
pub(crate) mod audio;
pub(crate) mod canvas_thumbnail;
pub(crate) mod commands;
pub(crate) mod enrichment;
pub(crate) mod export;
pub(crate) mod external_open;
pub(crate) mod files;
pub(crate) mod image;
pub(crate) mod import;
pub(crate) mod kind;
pub(crate) mod media_analysis;
pub(crate) mod node_asset;
pub(crate) mod node_payload;
pub(crate) mod reclamation;
mod recovery;
mod references;
pub(crate) mod transcription;
mod transfer;

use std::path::Path;

use serde::Serialize;

use super::knowledge::indexing::KnowledgeChunkSeed;
use crate::file_inspection::InspectedFile;
use kind::{AssetKind, PreparedAssetState};

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct PreparedAsset {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) path: Option<String>,
    pub(crate) kind: AssetKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) preview_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) preview_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) preview_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) extracted_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) modified_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) source_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) processing_state: Option<PreparedAssetState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) intrinsic_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) intrinsic_height: Option<u32>,
    #[serde(skip)]
    #[ts(skip)]
    pub(crate) knowledge_chunks: Option<Vec<KnowledgeChunkSeed>>,
    #[serde(skip)]
    #[ts(skip)]
    pub(crate) document_coverage: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) entries: Option<Vec<crate::file_inspection::ArchiveMember>>,
}

/// Shared post-acquisition mapping for imported copies, recordings and resource inspection.
/// Audio analysis is a durable node job, not document/thumbnail enrichment.
fn snapshot_to_imported(
    snapshot: InspectedFile,
    revision: String,
) -> Result<PreparedAsset, String> {
    let kind = AssetKind::from_node_kind(snapshot.kind)?;
    let geometry = if kind == AssetKind::Image {
        import::geometry::visual_geometry(Path::new(&snapshot.path), "image")?
    } else {
        None
    };
    Ok(PreparedAsset {
        name: snapshot.name,
        path: Some(snapshot.path),
        kind,
        text: None,
        preview_text: snapshot.preview_text,
        preview_language: snapshot.preview_language,
        preview_path: None,
        extracted_text: None,
        size: Some(snapshot.size),
        modified_at: Some(snapshot.modified_at),
        source_revision: Some(revision),
        processing_state: None,
        intrinsic_width: geometry.map(|value| value.0),
        intrinsic_height: geometry.map(|value| value.1),
        knowledge_chunks: None,
        document_coverage: None,
        entries: snapshot.entries,
    })
}

pub(crate) use access::asset_directory;

pub(crate) use access::{
    authorize_canvas_thumbnail, authorize_destination, authorize_edited_image_preview,
    authorize_media_source, authorize_source, authorize_spaces_asset,
};
pub(crate) use reclamation::{
    referenced_paths_for_nodes, referenced_paths_for_space, unreferenced_paths,
    unreferenced_paths_in_transaction,
};
pub(crate) use references::{
    rewrite_node_asset_paths, visit_asset_references, AssetReference, PACKAGE_ASSET_PREFIX,
};

pub(crate) async fn reconcile_interrupted_work(app: &tauri::AppHandle) -> Result<(), String> {
    recovery::reconcile(app)
        .await
        .map_err(|error| error.to_string())
}
