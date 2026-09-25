use serde::Deserialize;

use super::{
    PackageAssistantConversationRow, PackageAssistantMessageRow, PackageChunkRow,
    PackageDerivationRow, PackageEventRow, PackageExternalAgentConversationRow,
    PackageExternalAgentTurnRow, PackageNodeRow, PackageObjectTypeRow, PackageProvenanceRow,
    PackageRelationRow, PackageRunAttemptRow, PackageRunEventRow, PackageRunRow, PackageRunStepRow,
    PackageSceneRow, PackageSpaceRow,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestorePackageRequest {
    pub space: PackageSpaceRow,
    pub structure_nodes: Vec<crate::spaces::structure::StructureNode>,
    pub layouts: Vec<crate::spaces::structure::StructureLayout>,
    pub relation_styles: Vec<crate::spaces::structure::RelationStyle>,
    pub object_types: Vec<PackageObjectTypeRow>,
    pub nodes: Vec<PackageNodeRow>,
    pub scenes: Vec<PackageSceneRow>,
    pub relations: Vec<PackageRelationRow>,
    pub derivations: Vec<PackageDerivationRow>,
    pub runs: Vec<PackageRunRow>,
    pub run_steps: Vec<PackageRunStepRow>,
    pub run_attempts: Vec<PackageRunAttemptRow>,
    pub run_events: Vec<PackageRunEventRow>,
    pub external_agent_conversations: Vec<PackageExternalAgentConversationRow>,
    pub external_agent_turns: Vec<PackageExternalAgentTurnRow>,
    pub assistant_conversations: Vec<PackageAssistantConversationRow>,
    pub assistant_messages: Vec<PackageAssistantMessageRow>,
    pub provenance: Vec<PackageProvenanceRow>,
    pub knowledge_events: Vec<PackageEventRow>,
    pub chunks: Vec<PackageChunkRow>,
}
