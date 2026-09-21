mod access;
pub(crate) mod audio;
pub(crate) mod commands;
pub(crate) mod enrichment;
pub(crate) mod external_open;
pub(crate) mod files;
pub(crate) mod image;
pub(crate) mod import;
pub(crate) mod media_analysis;
mod reclamation;
mod recovery;
mod references;
pub(crate) mod transcription;
mod transfer;

use std::path::Path;

use serde::Serialize;

use super::knowledge::indexing::KnowledgeChunkSeed;
use crate::file_inspection::InspectedFile;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedAsset {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    pub(crate) kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) preview_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) preview_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) preview_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) extracted_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) modified_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) processing_state: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) intrinsic_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) intrinsic_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) knowledge_chunks: Option<Vec<KnowledgeChunkSeed>>,
    #[serde(skip)]
    pub(crate) document_coverage: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) entries: Option<Vec<crate::file_inspection::InspectedEntry>>,
}

/// Shared post-acquisition mapping for imported copies, recordings and resource inspection.
/// Audio analysis is a durable node job, not document/thumbnail enrichment.
fn snapshot_to_imported(
    snapshot: InspectedFile,
    revision: String,
) -> Result<PreparedAsset, String> {
    let geometry = if snapshot.kind == "image" {
        import::geometry::visual_geometry(Path::new(&snapshot.path), "image")?
    } else {
        None
    };
    Ok(PreparedAsset {
        name: snapshot.name,
        path: Some(snapshot.path),
        kind: snapshot.kind,
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
    authorize_canvas_thumbnail, authorize_destination, authorize_source, authorize_spaces_asset,
};
pub(crate) use reclamation::{
    reclaim_orphan_managed_assets, referenced_paths_for_nodes, referenced_paths_for_space,
    unreferenced_paths,
};
pub(crate) use references::{
    rewrite_node_asset_paths, visit_asset_references, AssetReference, PACKAGE_ASSET_PREFIX,
};

pub(crate) async fn reconcile_interrupted_work(app: &tauri::AppHandle) -> Result<(), String> {
    recovery::reconcile(app).await
}
