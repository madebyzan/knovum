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
    QoderCn,
    KimiCli,
    AuggieCli,
    Cline,
    Cursor,
    Antigravity,
    GrokBuild,
    Goose,
    OpenInterpreter,
    Trae,
    #[serde(rename = "openhands")]
    OpenHands,
    Junie,
    MistralVibe,
    Codebuddy,
    #[serde(rename = "minimax_code")]
    MiniMaxCode,
    PiAcp,
    Hermes,
    #[serde(rename = "openclaw")]
    OpenClaw,
    Devin,
    GlmAgent,
    Amp,
    FactoryDroid,
    Poolside,
    Bob,
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
            Self::QoderCn => "qoder_cn",
            Self::KimiCli => "kimi_cli",
            Self::AuggieCli => "auggie_cli",
            Self::Cline => "cline",
            Self::Cursor => "cursor",
            Self::Antigravity => "antigravity",
            Self::GrokBuild => "grok_build",
            Self::Goose => "goose",
            Self::OpenInterpreter => "open_interpreter",
            Self::Trae => "trae",
            Self::OpenHands => "openhands",
            Self::Junie => "junie",
            Self::MistralVibe => "mistral_vibe",
            Self::Codebuddy => "codebuddy",
            Self::MiniMaxCode => "minimax_code",
            Self::PiAcp => "pi_acp",
            Self::Hermes => "hermes",
            Self::OpenClaw => "openclaw",
            Self::Devin => "devin",
            Self::GlmAgent => "glm_agent",
            Self::Amp => "amp",
            Self::FactoryDroid => "factory_droid",
            Self::Poolside => "poolside",
            Self::Bob => "bob",
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
