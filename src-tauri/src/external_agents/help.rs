use crate::ai::contracts::{AgentConnectionKind, AgentHelp, AgentKind};

/// Vendor guidance is part of the Agent directory, not a settings-only lookup table.
pub(super) const fn describe(kind: AgentKind, connection: AgentConnectionKind) -> AgentHelp {
    let (documentation_url, setup_url) = match kind {
        AgentKind::Codex => (
            "https://github.com/agentclientprotocol/codex-acp",
            "https://github.com/agentclientprotocol/registry/blob/main/codex-acp/agent.json",
        ),
        AgentKind::ClaudeCode => (
            "https://github.com/agentclientprotocol/claude-agent-acp",
            "https://github.com/agentclientprotocol/registry/blob/main/claude-acp/agent.json",
        ),
        AgentKind::GeminiCli => (
            "https://geminicli.com",
            "https://github.com/agentclientprotocol/registry/blob/main/gemini/agent.json",
        ),
        AgentKind::GithubCopilot => (
            "https://github.com/github/copilot-cli",
            "https://github.com/agentclientprotocol/registry/blob/main/github-copilot-cli/agent.json",
        ),
        AgentKind::OpenCode => (
            "https://opencode.ai",
            "https://github.com/agentclientprotocol/registry/blob/main/opencode/agent.json",
        ),
        AgentKind::QwenCode => (
            "https://qwenlm.github.io/qwen-code-docs/en/users/overview",
            "https://github.com/agentclientprotocol/registry/blob/main/qwen-code/agent.json",
        ),
        AgentKind::QoderCli => (
            "https://qoder.com",
            "https://github.com/agentclientprotocol/registry/blob/main/qoder/agent.json",
        ),
        AgentKind::KimiCli => (
            "https://moonshotai.github.io/kimi-cli/",
            "https://github.com/agentclientprotocol/registry/blob/main/kimi/agent.json",
        ),
        AgentKind::AuggieCli => (
            "https://github.com/augmentcode/auggie",
            "https://github.com/agentclientprotocol/registry/blob/main/auggie/agent.json",
        ),
        AgentKind::Cline => (
            "https://cline.bot/cli",
            "https://github.com/agentclientprotocol/registry/blob/main/cline/agent.json",
        ),
        AgentKind::Cursor => (
            "https://cursor.com/docs/cli/acp",
            "https://github.com/agentclientprotocol/registry/blob/main/cursor/agent.json",
        ),
        AgentKind::Antigravity => (
            "https://antigravity.google/docs/ide/extensions",
            "https://github.com/agentclientprotocol/registry/blob/main/antigravity-acp/agent.json",
        ),
    };
    AgentHelp {
        connection,
        documentation_url,
        setup_url,
    }
}
