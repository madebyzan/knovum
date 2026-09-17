use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};

use super::{KnowledgeActionDefinition, KnowledgeActionParameter, KnowledgeActionParameterKind};

const PURPOSE: &[KnowledgeActionParameter] = &[text("purpose")];
const SCRIPT: &[KnowledgeActionParameter] = &[text("prompt")];
const SUMMARY: &[KnowledgeActionParameter] = &[text("summaryPurpose")];
const SOCIAL: &[KnowledgeActionParameter] = &[
    text("platform"),
    text("audience"),
    text("goal"),
    text("tone"),
];
const VISUALIZATION: &[KnowledgeActionParameter] =
    &[text("intent"), text("style"), text("aspectRatio")];
const COMPANION_IMAGE: &[KnowledgeActionParameter] = &[text("imageStyle"), text("aspectRatio")];
const VIDEO: &[KnowledgeActionParameter] = &[
    text("prompt"),
    text("aspectRatio"),
    number("durationSeconds"),
];

pub(super) const ACTIONS: &[KnowledgeActionDefinition] = &[
    action(
        "grounded-answer",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &["knowledge"],
        &[],
        "answer",
    ),
    action(
        "structure-document",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &["knowledge"],
        PURPOSE,
        "document",
    ),
    action(
        "write-script",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &["knowledge"],
        SCRIPT,
        "document",
    ),
    action(
        "summarize-meeting",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &["transcript"],
        SUMMARY,
        "document",
    ),
    action(
        "write-social-copy",
        Some(ExecutionCapability::TextGeneration),
        Some(RoutingPurpose::Knowledge),
        &["knowledge"],
        SOCIAL,
        "document",
    ),
    action(
        "knowledge-visualization",
        Some(ExecutionCapability::ImageGeneration),
        Some(RoutingPurpose::ImageGeneration),
        &["knowledge"],
        VISUALIZATION,
        "image",
    ),
    action(
        "create-companion-image",
        Some(ExecutionCapability::ImageGeneration),
        Some(RoutingPurpose::ImageGeneration),
        &["document"],
        COMPANION_IMAGE,
        "image",
    ),
    action(
        "generate-video",
        Some(ExecutionCapability::VideoGeneration),
        Some(RoutingPurpose::VideoGeneration),
        &["document"],
        VIDEO,
        "video",
    ),
    action("extract-subtitles", None, None, &["video"], &[], "subtitle"),
    action(
        "render-subtitled-video",
        None,
        None,
        &["video", "subtitle"],
        &[],
        "video",
    ),
    action(
        "render-presentation",
        None,
        None,
        &["document"],
        &[],
        "presentation",
    ),
];

const fn action(
    id: &'static str,
    execution_capability: Option<ExecutionCapability>,
    routing_purpose: Option<RoutingPurpose>,
    input_roles: &'static [&'static str],
    parameters: &'static [KnowledgeActionParameter],
    output_contract: &'static str,
) -> KnowledgeActionDefinition {
    KnowledgeActionDefinition {
        id,
        version: 1,
        execution_capability,
        routing_purpose,
        input_roles,
        parameters,
        output_contract,
    }
}

const fn text(key: &'static str) -> KnowledgeActionParameter {
    KnowledgeActionParameter {
        key,
        kind: KnowledgeActionParameterKind::Text,
    }
}

const fn number(key: &'static str) -> KnowledgeActionParameter {
    KnowledgeActionParameter {
        key,
        kind: KnowledgeActionParameterKind::Number,
    }
}
