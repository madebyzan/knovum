use super::TaskTargetSelection;
use crate::ai::models::EditorTransformAction;
use serde::Deserialize;

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorTransformCommandRequest {
    pub target: TaskTargetSelection,
    pub action: EditorTransformAction,
    pub text: String,
}
#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TranslationCommandRequest {
    pub execution_target: TaskTargetSelection,
    pub text: String,
    pub target_language: String,
}
