mod definition;
pub(crate) mod parameters;

use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};

use super::{
    SpaceActionCollectionInput, SpaceActionDefinition, SpaceActionInputRole as Input,
    SpaceActionOutputContract as Output,
};
use definition::{
    action, durable_query_action, immediate_action, immediate_mutation_action, query_action,
};
use parameters::{
    contents, empty, image, image_transform, knowledge_changes, meeting_summary, node, note,
    purpose, question, script, search, social_copy, space_edit, video, visualization,
};

pub(super) const ACTIONS: &[SpaceActionDefinition] = &[
    action(
        "upscale-image",
        None,
        None,
        &[Input::Image],
        image_transform,
        Output::Image,
    ),
    action(
        "generate-image-angle",
        None,
        None,
        &[Input::Image],
        image_transform,
        Output::Image,
    ),
    action(
        "remove-image-object",
        None,
        None,
        &[Input::Image],
        image_transform,
        Output::Image,
    ),
    action(
        "remove-image-background",
        None,
        None,
        &[Input::Image],
        image_transform,
        Output::Image,
    ),
    action(
        "extract-image-selection",
        None,
        None,
        &[Input::Image],
        image_transform,
        Output::Image,
    ),
    action(
        "generate-image-depth",
        None,
        None,
        &[Input::Image],
        image_transform,
        Output::Image,
    ),
    action(
        "split-image-layers",
        None,
        None,
        &[Input::Image],
        empty,
        Output::Image,
    ),
    immediate_action("create-note", &[], note, Output::ResourceReceipt),
    immediate_action(
        "commit-knowledge-changes",
        &[],
        knowledge_changes,
        Output::KnowledgeChangeReceipt,
    ),
    immediate_mutation_action("edit-space-structure", space_edit, Output::SpaceEditReceipt),
    query_action(
        "list-space-contents",
        None,
        None,
        &[],
        contents,
        Output::SpaceContents,
    ),
    query_action(
        "search-knowledge",
        None,
        None,
        &[],
        search,
        Output::SearchResults,
    ),
    query_action("read-node", None, None, &[], node, Output::KnowledgeNode),
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
    collection_action(
        action(
            "generate-video",
            Some(ExecutionCapability::VideoGeneration),
            Some(RoutingPurpose::VideoGeneration),
            &[Input::Knowledge],
            video,
            Output::Video,
        ),
        SpaceActionCollectionInput {
            kind: "image",
            role: "source",
            max_items: 10,
        },
    ),
    action(
        "add-video-subtitles",
        None,
        None,
        &[Input::Video],
        empty,
        Output::Video,
    ),
    action(
        "convert-document-to-docx",
        None,
        None,
        &[Input::Document],
        empty,
        Output::File,
    ),
    action(
        "convert-document-to-pdf",
        None,
        None,
        &[Input::Document],
        empty,
        Output::File,
    ),
    action(
        "convert-document-to-txt",
        None,
        None,
        &[Input::Document],
        empty,
        Output::File,
    ),
    action(
        "convert-document-to-html",
        None,
        None,
        &[Input::Document],
        empty,
        Output::File,
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

const fn collection_action(
    mut definition: SpaceActionDefinition,
    collection: SpaceActionCollectionInput,
) -> SpaceActionDefinition {
    definition.collection_input = Some(collection);
    definition
}
