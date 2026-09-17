use serde::Deserialize;

use super::{
    PackageCanvasState, PackageChunkRow, PackageDerivationRow, PackageEventRow, PackageMessageRow,
    PackageNodeRow, PackageObjectTypeRow, PackageProvenanceRow, PackageRelationRow,
    PackageSceneRow, PackageSpaceRow, PackageThreadRow, PackageWorkflowRunRow,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestorePackageRequest {
    pub space: PackageSpaceRow,
    pub canvas_state: PackageCanvasState,
    pub object_types: Vec<PackageObjectTypeRow>,
    pub nodes: Vec<PackageNodeRow>,
    pub scenes: Vec<PackageSceneRow>,
    pub relations: Vec<PackageRelationRow>,
    pub derivations: Vec<PackageDerivationRow>,
    pub creation_workflow_runs: Vec<PackageWorkflowRunRow>,
    pub provenance: Vec<PackageProvenanceRow>,
    pub knowledge_events: Vec<PackageEventRow>,
    pub chunks: Vec<PackageChunkRow>,
    pub threads: Vec<PackageThreadRow>,
    pub messages: Vec<PackageMessageRow>,
}
