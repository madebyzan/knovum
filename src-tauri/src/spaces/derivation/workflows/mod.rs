use serde::Serialize;
use tauri::AppHandle;

mod saved;

#[cfg(test)]
mod tests;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStepDefinition {
    pub id: &'static str,
    pub action: &'static str,
    pub depends_on: &'static [&'static str],
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinition {
    pub id: &'static str,
    pub version: u16,
    pub steps: &'static [WorkflowStepDefinition],
    pub parameters: &'static [WorkflowParameterDefinition],
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowParameterKind {
    Text,
    Number,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowParameterDefinition {
    pub key: &'static str,
    pub kind: WorkflowParameterKind,
}

const KNOWLEDGE_PRESENTATION: &[WorkflowStepDefinition] = &[
    step("document", "structure-document", &[]),
    step("presentation", "render-presentation", &["document"]),
];
const KNOWLEDGE_VIDEO: &[WorkflowStepDefinition] = &[
    step("script", "write-script", &[]),
    step("video", "generate-video", &["script"]),
];
const MEETING_VIDEO: &[WorkflowStepDefinition] = &[
    step("subtitles", "extract-subtitles", &[]),
    step("summary", "summarize-meeting", &[]),
    step("video", "render-subtitled-video", &["subtitles"]),
];
const SOCIAL_CONTENT: &[WorkflowStepDefinition] = &[
    step("copy", "write-social-copy", &[]),
    step("image", "create-companion-image", &["copy"]),
];
const VIDEO_SUBTITLES: &[WorkflowStepDefinition] = &[
    step("subtitles", "extract-subtitles", &[]),
    step("video", "render-subtitled-video", &["subtitles"]),
];
const PRESENTATION_PARAMETERS: &[WorkflowParameterDefinition] = &[text_parameter("purpose")];
const VIDEO_PARAMETERS: &[WorkflowParameterDefinition] = &[
    text_parameter("prompt"),
    text_parameter("aspectRatio"),
    number_parameter("durationSeconds"),
];
const MEETING_PARAMETERS: &[WorkflowParameterDefinition] = &[text_parameter("summaryPurpose")];
const SOCIAL_PARAMETERS: &[WorkflowParameterDefinition] = &[
    text_parameter("platform"),
    text_parameter("audience"),
    text_parameter("goal"),
    text_parameter("tone"),
    text_parameter("imageStyle"),
    text_parameter("aspectRatio"),
];
const WORKFLOWS: &[WorkflowDefinition] = &[
    workflow(
        "knowledge-presentation",
        KNOWLEDGE_PRESENTATION,
        PRESENTATION_PARAMETERS,
    ),
    workflow("knowledge-video", KNOWLEDGE_VIDEO, VIDEO_PARAMETERS),
    workflow("meeting-video-package", MEETING_VIDEO, MEETING_PARAMETERS),
    workflow("social-content-kit", SOCIAL_CONTENT, SOCIAL_PARAMETERS),
    workflow("video-with-subtitles", VIDEO_SUBTITLES, &[]),
];

const fn step(
    id: &'static str,
    action: &'static str,
    depends_on: &'static [&'static str],
) -> WorkflowStepDefinition {
    WorkflowStepDefinition {
        id,
        action,
        depends_on,
    }
}

const fn text_parameter(key: &'static str) -> WorkflowParameterDefinition {
    WorkflowParameterDefinition {
        key,
        kind: WorkflowParameterKind::Text,
    }
}

const fn number_parameter(key: &'static str) -> WorkflowParameterDefinition {
    WorkflowParameterDefinition {
        key,
        kind: WorkflowParameterKind::Number,
    }
}

const fn workflow(
    id: &'static str,
    steps: &'static [WorkflowStepDefinition],
    parameters: &'static [WorkflowParameterDefinition],
) -> WorkflowDefinition {
    WorkflowDefinition {
        id,
        version: 1,
        steps,
        parameters,
    }
}

pub fn require(id: &str, version: i64) -> Result<&'static WorkflowDefinition, String> {
    let definition = WORKFLOWS
        .iter()
        .find(|definition| definition.id == id && i64::from(definition.version) == version)
        .ok_or_else(|| format!("Creation Workflow is not registered: {id}@{version}"))?;
    Ok(definition)
}

#[tauri::command]
pub fn list_creation_workflows() -> Vec<WorkflowDefinition> {
    WORKFLOWS.to_vec()
}

#[tauri::command]
pub fn validate_creation_library(
    app: AppHandle,
    generation_presets: Vec<saved::SavedGenerationPreset>,
    workflows: Vec<saved::SavedWorkflow>,
) -> Result<(), String> {
    saved::validate_presets(&app, generation_presets)?;
    saved::validate_workflows(&app, workflows)
}
