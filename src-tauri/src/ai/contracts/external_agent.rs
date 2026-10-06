use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAgentKind {
    Codex,
    ClaudeCode,
    GeminiCli,
    GithubCopilot,
    OpenCode,
    QwenCode,
    QoderCli,
    KimiCli,
    AuggieCli,
    Cline,
    Cursor,
    Antigravity,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentDescriptor {
    pub kind: ExternalAgentKind,
    pub display_name: &'static str,
    pub executable_names: &'static [&'static str],
    pub help: ExternalAgentHelp,
}

#[derive(Debug, Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAgentConnectionKind {
    ExecutableCli,
    NodeAdapter,
    StandaloneServer,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentHelp {
    pub connection: ExternalAgentConnectionKind,
    pub documentation_url: &'static str,
    pub setup_url: &'static str,
}

impl ExternalAgentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude_code",
            Self::GeminiCli => "gemini_cli",
            Self::GithubCopilot => "github_copilot",
            Self::OpenCode => "open_code",
            Self::QwenCode => "qwen_code",
            Self::QoderCli => "qoder_cli",
            Self::KimiCli => "kimi_cli",
            Self::AuggieCli => "auggie_cli",
            Self::Cline => "cline",
            Self::Cursor => "cursor",
            Self::Antigravity => "antigravity",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentInstallation {
    pub kind: ExternalAgentKind,
    /// A registered desktop application or configured execution entry was found.
    pub detected: bool,
    /// ACP host/server entry only. Never a desktop application's GUI executable.
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum AgentArtifactKind {
    Image,
    Video,
    Audio,
    Document,
    File,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AgentArtifact {
    pub name: String,
    pub relative_path: String,
    pub kind: AgentArtifactKind,
    pub size: u64,
}
