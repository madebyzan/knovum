use super::TaskTargetSelection;
use crate::ai::models::EditorTransformAction;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorContextReference {
    pub node_id: String,
    pub title: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Default, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorDocumentContext {
    #[serde(default)]
    #[ts(optional)]
    pub document_title: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub section_heading: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub preceding_text: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub following_text: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub references: Option<Vec<EditorContextReference>>,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorTransformCommandRequest {
    pub target: TaskTargetSelection,
    pub action: EditorTransformAction,
    pub text: String,
    #[serde(default)]
    #[ts(optional)]
    pub instruction: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub context: Option<EditorDocumentContext>,
    #[serde(default)]
    #[ts(optional)]
    pub previous_draft: Option<String>,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TranslationCommandRequest {
    pub execution_target: TaskTargetSelection,
    pub text: String,
    pub target_language: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum NarrationPreparationMode {
    Clean,
    Adapt,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationPreparationCommandRequest {
    pub target: TaskTargetSelection,
    pub mode: NarrationPreparationMode,
    pub text: String,
}
