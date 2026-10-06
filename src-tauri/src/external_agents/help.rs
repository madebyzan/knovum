use crate::ai::contracts::{ExternalAgentConnectionKind, ExternalAgentHelp, ExternalAgentKind};

/// Vendor guidance is part of the Agent directory, not a settings-only lookup table.
pub(super) const fn describe(
    kind: ExternalAgentKind,
    connection: ExternalAgentConnectionKind,
) -> ExternalAgentHelp {
    let (documentation_url, setup_url) = match kind {
        ExternalAgentKind::Codex => (
            "https://github.com/agentclientprotocol/codex-acp",
            "https://github.com/agentclientprotocol/registry/blob/main/codex-acp/agent.json",
        ),
        ExternalAgentKind::ClaudeCode => (
            "https://github.com/agentclientprotocol/claude-agent-acp",
            "https://github.com/agentclientprotocol/registry/blob/main/claude-acp/agent.json",
        ),
        ExternalAgentKind::GeminiCli => (
            "https://geminicli.com",
            "https://github.com/agentclientprotocol/registry/blob/main/gemini/agent.json",
        ),
        ExternalAgentKind::GithubCopilot => (
            "https://github.com/github/copilot-cli",
            "https://github.com/agentclientprotocol/registry/blob/main/github-copilot-cli/agent.json",
        ),
        ExternalAgentKind::OpenCode => (
            "https://opencode.ai",
            "https://github.com/agentclientprotocol/registry/blob/main/opencode/agent.json",
        ),
        ExternalAgentKind::QwenCode => (
            "https://qwenlm.github.io/qwen-code-docs/en/users/overview",
            "https://github.com/agentclientprotocol/registry/blob/main/qwen-code/agent.json",
        ),
        ExternalAgentKind::QoderCli => (
            "https://qoder.com",
            "https://github.com/agentclientprotocol/registry/blob/main/qoder/agent.json",
        ),
        ExternalAgentKind::KimiCli => (
            "https://moonshotai.github.io/kimi-cli/",
            "https://github.com/agentclientprotocol/registry/blob/main/kimi/agent.json",
        ),
        ExternalAgentKind::AuggieCli => (
            "https://github.com/augmentcode/auggie",
            "https://github.com/agentclientprotocol/registry/blob/main/auggie/agent.json",
        ),
        ExternalAgentKind::Cline => (
            "https://cline.bot/cli",
            "https://github.com/agentclientprotocol/registry/blob/main/cline/agent.json",
        ),
        ExternalAgentKind::Cursor => (
            "https://cursor.com/docs/cli/acp",
            "https://github.com/agentclientprotocol/registry/blob/main/cursor/agent.json",
        ),
        ExternalAgentKind::Antigravity => (
            "https://antigravity.google/docs/ide/extensions",
            "https://github.com/agentclientprotocol/registry/blob/main/antigravity-acp/agent.json",
        ),
    };
    ExternalAgentHelp {
        connection,
        documentation_url,
        setup_url,
    }
}
