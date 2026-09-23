use serde::Deserialize;

use super::{
    PackageAssistantConversationRow, PackageAssistantMessageRow, PackageAutomationOccurrenceRow,
    PackageAutomationTriggerRow, PackageChunkRow, PackageDerivationRow, PackageEventRow,
    PackageExternalAgentConversationRow, PackageExternalAgentMessageRow,
    PackageExternalAgentTurnRow, PackageExternalToolBindingRow, PackageMessageRow, PackageNodeRow,
    PackageObjectTypeRow, PackageProvenanceRow, PackageRelationRow, PackageRunAttemptRow,
    PackageRunBatchItemRow, PackageRunEventRow, PackageRunRow, PackageRunStepRow, PackageSceneRow,
    PackageSpaceRow, PackageThreadRow, PackageWorkflowDraftRow, PackageWorkflowVersionRow,
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
    pub workflow_drafts: Vec<PackageWorkflowDraftRow>,
    pub workflow_versions: Vec<PackageWorkflowVersionRow>,
    pub runs: Vec<PackageRunRow>,
    pub run_steps: Vec<PackageRunStepRow>,
    pub run_batch_items: Vec<PackageRunBatchItemRow>,
    pub run_attempts: Vec<PackageRunAttemptRow>,
    pub run_events: Vec<PackageRunEventRow>,
    pub automation_triggers: Vec<PackageAutomationTriggerRow>,
    pub automation_occurrences: Vec<PackageAutomationOccurrenceRow>,
    pub external_tool_bindings: Vec<PackageExternalToolBindingRow>,
    pub external_agent_conversations: Vec<PackageExternalAgentConversationRow>,
    pub external_agent_turns: Vec<PackageExternalAgentTurnRow>,
    pub external_agent_messages: Vec<PackageExternalAgentMessageRow>,
    pub assistant_conversations: Vec<PackageAssistantConversationRow>,
    pub assistant_messages: Vec<PackageAssistantMessageRow>,
    pub provenance: Vec<PackageProvenanceRow>,
    pub knowledge_events: Vec<PackageEventRow>,
    pub chunks: Vec<PackageChunkRow>,
    pub threads: Vec<PackageThreadRow>,
    pub messages: Vec<PackageMessageRow>,
}
