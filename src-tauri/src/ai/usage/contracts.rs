use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ts_rs::TS, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsagePurpose {
    BuiltinAgent,
    Editor,
    Knowledge,
    KnowledgeIndexing,
    Translation,
    VisualAnalysis,
}

impl UsagePurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BuiltinAgent => "builtin_agent",
            Self::Editor => "editor",
            Self::Knowledge => "knowledge",
            Self::KnowledgeIndexing => "knowledge_indexing",
            Self::Translation => "translation",
            Self::VisualAnalysis => "visual_analysis",
        }
    }
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageReportRequest {
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UsageDay {
    pub date: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub requests: u64,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UsagePurposeTotal {
    pub purpose: UsagePurpose,
    pub total_tokens: u64,
    pub requests: u64,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub requests: u64,
    pub active_days: u64,
    pub peak_daily_tokens: u64,
    pub current_streak_days: u64,
    pub longest_streak_days: u64,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub from_date: String,
    pub to_date: String,
    pub summary: UsageMetrics,
    pub activity: Vec<UsageDay>,
    pub by_purpose: Vec<UsagePurposeTotal>,
}
