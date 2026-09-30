use serde::{Deserialize, Serialize};

use super::ModelExecutionTarget;

#[derive(Debug)]
pub struct CompletionRequest {
    pub purpose: crate::ai::usage::UsagePurpose,
    pub target: ModelExecutionTarget,
    pub system_prompt: String,
    pub instruction: String,
    pub text: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Copy, Deserialize, ts_rs::TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CompletionOutput {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub usage: Option<crate::ai::contracts::TokenUsage>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
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
