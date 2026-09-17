use serde::{Deserialize, Serialize};

use super::{ExecutionTarget, ExecutionTargetSelection};

#[derive(Debug)]
pub struct CompletionRequest {
    pub target: ExecutionTarget,
    pub system_prompt: String,
    pub instruction: String,
    pub text: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletionCommandRequest {
    pub target: ExecutionTargetSelection,
    pub system_prompt: String,
    pub instruction: String,
    pub text: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorTransformAction {
    Improve,
    FixGrammar,
    Extend,
    Reduce,
    Simplify,
    Emojify,
    Complete,
    Summarize,
    ToneAcademic,
    ToneBusiness,
    ToneCasual,
    ToneConfident,
    ToneCreative,
    ToneFriendly,
    ToneHumorous,
    ToneInformative,
    ToneInspirational,
    ToneObjective,
    TonePersuasive,
}

#[derive(Debug)]
pub struct EditorTransformRequest {
    pub target: ExecutionTarget,
    pub action: EditorTransformAction,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditorTransformCommandRequest {
    pub target: ExecutionTargetSelection,
    pub action: EditorTransformAction,
    pub text: String,
}

#[derive(Debug)]
pub struct TranslationRequest {
    pub execution_target: ExecutionTarget,
    pub text: String,
    pub target_language: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TranslationCommandRequest {
    pub execution_target: ExecutionTargetSelection,
    pub text: String,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionOutput {
    pub text: String,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum CompletionStreamEvent {
    Started {
        provider: String,
        model: String,
    },
    ThinkingDelta {
        text: String,
    },
    Delta {
        text: String,
    },
    Usage {
        prompt_tokens: u32,
        completion_tokens: u32,
        total_tokens: u32,
    },
}
