use serde::Deserialize;

use super::{
    PackageAssistantConversation, PackageAssistantMessage, PackageChunk, PackageDerivation,
    PackageEvent, PackageExternalAgentConversation, PackageExternalAgentTurn, PackageNode,
    PackageObjectType, PackageProvenance, PackageRelation, PackageRun, PackageRunAttempt,
    PackageRunEvent, PackageRunStep, PackageScene, PackageSpace, PackageSpeechProject,
    PackageSpeechProjectRevision, PackageSpeechRender, PackageSpeechSegment, PackageVoiceProfile,
    PackageVoiceRevision,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestorePackageRequest {
    pub space: PackageSpace,
    pub structure_nodes: Vec<crate::spaces::structure::StructureNode>,
    pub layouts: Vec<crate::spaces::structure::StructureLayout>,
    pub relation_styles: Vec<crate::spaces::structure::RelationStyle>,
    pub object_types: Vec<PackageObjectType>,
    pub nodes: Vec<PackageNode>,
    pub scenes: Vec<PackageScene>,
    pub relations: Vec<PackageRelation>,
    pub derivations: Vec<PackageDerivation>,
    pub runs: Vec<PackageRun>,
    pub run_steps: Vec<PackageRunStep>,
    pub run_attempts: Vec<PackageRunAttempt>,
    pub run_events: Vec<PackageRunEvent>,
    pub external_agent_conversations: Vec<PackageExternalAgentConversation>,
    pub external_agent_turns: Vec<PackageExternalAgentTurn>,
    pub assistant_conversations: Vec<PackageAssistantConversation>,
    pub assistant_messages: Vec<PackageAssistantMessage>,
    pub voice_profiles: Vec<PackageVoiceProfile>,
    pub voice_revisions: Vec<PackageVoiceRevision>,
    pub speech_projects: Vec<PackageSpeechProject>,
    pub speech_project_revisions: Vec<PackageSpeechProjectRevision>,
    pub speech_segments: Vec<PackageSpeechSegment>,
    pub speech_renders: Vec<PackageSpeechRender>,
    pub provenance: Vec<PackageProvenance>,
    pub knowledge_events: Vec<PackageEvent>,
    pub chunks: Vec<PackageChunk>,
}
