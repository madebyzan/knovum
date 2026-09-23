mod definition;
mod parameters;

use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};

use super::{
    SpaceActionDefinition, SpaceActionInputRole as Input, SpaceActionOutputContract as Output,
};
use definition::{
    action, durable_query_action, immediate_action, immediate_mutation_action, query_action,
};
use parameters::*;

pub(super) const ACTIONS: &[SpaceActionDefinition] = &[
    immediate_action("create-note", &[], note, Output::ResourceReceipt),
    immediate_action(
        "commit-knowledge-changes",
        &[],
        knowledge_changes,
        Output::KnowledgeChangeReceipt,
    ),
    immediate_mutation_action("edit-space-structure", space_edit, Output::SpaceEditReceipt),
    immediate_mutation_action("save-workflow-draft", workflow_draft, Output::WorkflowDraft),
    immediate_mutation_action(
        "publish-workflow",
        workflow_publication,
        Output::WorkflowVersion,
    ),
    query_action(
        "search-knowledge",
        None,
        None,
        &[Input::Knowledge],
        search,
        Output::SearchResults,
    ),
    query_action(
        "read-node",
        None,
        None,
        &[Input::Knowledge],
        node,
        Output::KnowledgeNode,
    ),
    durable_query_action(
        "answer-with-sources",
        ExecutionCapability::TextGeneration,
        RoutingPurpose::Knowledge,
        &[Input::Knowledge],
        question,
        Output::Answer,
    ),
    action(
        "draft-document",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &[Input::Knowledge],
        purpose,
        Output::Document,
    ),
    action(
        "draft-script",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &[Input::Knowledge],
        script,
        Output::Document,
    ),
    action(
        "summarize-meeting",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &[Input::Transcript],
        meeting_summary,
        Output::Document,
    ),
    action(
        "draft-social-post",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &[Input::Knowledge],
        social_copy,
        Output::Document,
    ),
    action(
        "generate-visualization",
        Some(ExecutionCapability::ImageGeneration),
        Some(RoutingPurpose::ImageGeneration),
        &[Input::Knowledge],
        visualization,
        Output::Image,
    ),
    action(
        "generate-image",
        Some(ExecutionCapability::ImageGeneration),
        Some(RoutingPurpose::ImageGeneration),
        &[Input::Document],
        image,
        Output::Image,
    ),
    action(
        "generate-video",
        Some(ExecutionCapability::VideoGeneration),
        Some(RoutingPurpose::VideoGeneration),
        &[Input::Document],
        video,
        Output::Video,
    ),
    action(
        "extract-subtitles",
        None,
        None,
        &[Input::Video],
        empty,
        Output::Subtitle,
    ),
    action(
        "render-subtitled-video",
        None,
        None,
        &[Input::Video, Input::Subtitle],
        empty,
        Output::Video,
    ),
    action(
        "render-presentation",
        None,
        None,
        &[Input::Document],
        empty,
        Output::Presentation,
    ),
];
