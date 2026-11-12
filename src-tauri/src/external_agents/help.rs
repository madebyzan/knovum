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
        ExternalAgentKind::QoderCn => ("https://qoder.cn", "https://docs.qoder.cn/cli/acp"),
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
        ExternalAgentKind::GrokBuild => (
            "https://x.ai/cli",
            "https://github.com/agentclientprotocol/registry/blob/main/grok-build/agent.json",
        ),
        ExternalAgentKind::Goose => (
            "https://block.github.io/goose",
            "https://github.com/agentclientprotocol/registry/blob/main/goose/agent.json",
        ),
        ExternalAgentKind::OpenInterpreter => (
            "https://openinterpreter.com",
            "https://openinterpreter.com/docs/terminal/acp",
        ),
        ExternalAgentKind::Trae => ("https://trae.ai", "https://docs.trae.ai/cli/acp"),
        ExternalAgentKind::OpenHands => (
            "https://github.com/OpenHands/OpenHands",
            "https://docs.openhands.dev/overview/quickstart",
        ),
        ExternalAgentKind::Junie => (
            "https://junie.jetbrains.com",
            "https://github.com/agentclientprotocol/registry/blob/main/junie/agent.json",
        ),
        ExternalAgentKind::MistralVibe => (
            "https://mistral.ai/products/vibe",
            "https://github.com/agentclientprotocol/registry/blob/main/mistral-vibe/agent.json",
        ),
        ExternalAgentKind::Codebuddy => (
            "https://www.codebuddy.cn/cli/",
            "https://github.com/agentclientprotocol/registry/blob/main/codebuddy-code/agent.json",
        ),
        ExternalAgentKind::MiniMaxCode => (
            "https://agent.minimax.io",
            "https://github.com/agentclientprotocol/registry/blob/main/minimax-code/agent.json",
        ),
        ExternalAgentKind::PiAcp => (
            "https://github.com/svkozak/pi-acp",
            "https://github.com/agentclientprotocol/registry/blob/main/pi-acp/agent.json",
        ),
        ExternalAgentKind::Hermes => (
            "https://hermes-agent.nousresearch.com",
            "https://hermes-agent.nousresearch.com/docs/user-guide/features/acp",
        ),
        ExternalAgentKind::OpenClaw => (
            "https://docs.openclaw.ai",
            "https://docs.openclaw.ai/cli/acp",
        ),
        ExternalAgentKind::Devin => (
            "https://docs.devin.ai/cli",
            "https://github.com/agentclientprotocol/registry/blob/main/devin/agent.json",
        ),
        ExternalAgentKind::GlmAgent => (
            "https://github.com/stefandevo/glm-acp-agent",
            "https://github.com/agentclientprotocol/registry/blob/main/glm-acp-agent/agent.json",
        ),
        ExternalAgentKind::Amp => (
            "https://github.com/tao12345666333/amp-acp",
            "https://github.com/agentclientprotocol/registry/blob/main/amp-acp/agent.json",
        ),
        ExternalAgentKind::FactoryDroid => (
            "https://factory.ai/product/cli",
            "https://github.com/agentclientprotocol/registry/blob/main/factory-droid/agent.json",
        ),
        ExternalAgentKind::Poolside => (
            "https://poolside.ai",
            "https://github.com/agentclientprotocol/registry/blob/main/poolside/agent.json",
        ),
        ExternalAgentKind::Bob => (
            "https://bob.ibm.com",
            "https://github.com/agentclientprotocol/registry/blob/main/bob/agent.json",
        ),
    };
    ExternalAgentHelp {
        connection,
        documentation_url,
        setup_url,
    }
}
