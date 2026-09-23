use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDescriptor {
    pub kind: AgentKind,
    pub display_name: &'static str,
    pub executable_names: &'static [&'static str],
    pub help: AgentHelp,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentConnectionKind {
    NativeCli,
    NodeAdapter,
    StandaloneServer,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentHelp {
    pub connection: AgentConnectionKind,
    pub documentation_url: &'static str,
    pub setup_url: &'static str,
}

impl AgentKind {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstallation {
    pub kind: AgentKind,
    /// A registered desktop application or configured execution entry was found.
    pub detected: bool,
    /// ACP host/server entry only. Never a desktop application's GUI executable.
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentArtifactKind {
    Image,
    Video,
    Audio,
    Document,
    File,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentArtifact {
    pub name: String,
    pub relative_path: String,
    pub kind: AgentArtifactKind,
    pub size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverableAgentCreation {
    pub session_id: String,
    pub artifacts: Vec<AgentArtifact>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentCreationRequest {
    pub kind: AgentKind,
    pub prompt: String,
    pub attachments: Vec<std::path::PathBuf>,
    pub space_id: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum AgentCreationEvent {
    Thought {
        text: String,
    },
    Message {
        text: String,
    },
    ToolStarted {
        id: String,
        title: String,
    },
    ToolUpdated {
        id: String,
    },
    PermissionRequested {
        request_id: String,
        title: String,
        options: Vec<AgentPermissionOption>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPermissionOption {
    pub id: String,
    pub label: String,
    pub kind: String,
}
