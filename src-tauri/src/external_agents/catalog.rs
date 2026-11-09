use crate::ai::contracts::{
    ExternalAgentConnectionKind, ExternalAgentDescriptor, ExternalAgentKind,
};

#[derive(Clone, Copy)]
pub(super) enum AcpLaunch {
    Codex,
    Claude,
    Executable {
        arguments: &'static [&'static str],
        env: &'static [(&'static str, &'static str)],
    },
    Server,
}

/// The npm adapters the Node-adapter Agents run through, pinned to exact versions. `npx`
/// resolves a tag such as `@latest` against the registry on every launch — a network round
/// trip before every answer — while an exact version already in its cache starts offline. The
/// pin is also what `trusted_session_mode` was checked against: an adapter that renames its
/// session modes reaches users only with the release that updates both.
pub(super) const CODEX_ADAPTER: &str = "@agentclientprotocol/codex-acp@2.6.0";
pub(super) const CLAUDE_ADAPTER: &str = "@agentclientprotocol/claude-agent-acp@0.101.1";

impl AcpLaunch {
    /// The launch facts (executable names, arguments, environment) mirror each Agent's entry
    /// in the ACP registry or its official ACP documentation; they are not a Knovum invention.
    pub const fn executable(arguments: &'static [&'static str]) -> Self {
        Self::Executable {
            arguments,
            env: &[],
        }
    }

    pub const fn executable_with_env(
        arguments: &'static [&'static str],
        env: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self::Executable { arguments, env }
    }

    pub const fn trusted_session_mode(self) -> Option<&'static str> {
        match self {
            Self::Codex => Some("agent-full-access"),
            Self::Claude => Some("bypassPermissions"),
            Self::Executable { .. } | Self::Server => None,
        }
    }

    const fn connection(self) -> ExternalAgentConnectionKind {
        match self {
            Self::Codex | Self::Claude => ExternalAgentConnectionKind::NodeAdapter,
            Self::Executable { .. } => ExternalAgentConnectionKind::ExecutableCli,
            Self::Server => ExternalAgentConnectionKind::StandaloneServer,
        }
    }
}

pub(super) struct RegisteredAgent {
    pub descriptor: ExternalAgentDescriptor,
    pub launch: AcpLaunch,
    pub desktop_client: Option<DesktopClient>,
}

pub(super) struct DesktopClient {
    #[cfg(target_os = "macos")]
    pub bundle_identifiers: &'static [&'static str],
    #[cfg(windows)]
    pub display_names: &'static [&'static str],
}

const fn agent(
    kind: ExternalAgentKind,
    display_name: &'static str,
    executable_names: &'static [&'static str],
    launch: AcpLaunch,
) -> RegisteredAgent {
    RegisteredAgent {
        descriptor: ExternalAgentDescriptor {
            kind,
            display_name,
            executable_names,
            help: super::help::describe(kind, launch.connection()),
        },
        launch,
        desktop_client: None,
    }
}

pub(super) const AGENTS: &[RegisteredAgent] = &[
    agent(
        ExternalAgentKind::Codex,
        "OpenAI Codex",
        &["codex"],
        AcpLaunch::Codex,
    ),
    agent(
        ExternalAgentKind::ClaudeCode,
        "Anthropic Claude Code",
        &["claude"],
        AcpLaunch::Claude,
    ),
    agent(
        ExternalAgentKind::GeminiCli,
        "Google Gemini CLI",
        &["gemini"],
        AcpLaunch::executable(&["--acp"]),
    ),
    agent(
        ExternalAgentKind::GithubCopilot,
        "GitHub Copilot",
        &["copilot"],
        AcpLaunch::executable(&["--acp"]),
    ),
    agent(
        ExternalAgentKind::OpenCode,
        "OpenCode",
        &["opencode"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::QwenCode,
        "Qwen Code",
        &["qwen"],
        AcpLaunch::executable(&["--acp", "--experimental-skills"]),
    ),
    agent(
        ExternalAgentKind::QoderCli,
        "Qoder CLI",
        &["qodercli", "qoder"],
        AcpLaunch::executable(&["--acp"]),
    ),
    // The China edition ships as its own CLI (@qodercn-ai/qoderclicn) with distinct
    // binaries; qoder.cn docs specify `qodercn --acp` for ACP clients.
    agent(
        ExternalAgentKind::QoderCn,
        "Qoder CLI CN",
        &["qodercn", "qoderclicn"],
        AcpLaunch::executable(&["--acp"]),
    ),
    agent(
        ExternalAgentKind::KimiCli,
        "Kimi CLI",
        &["kimi"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::AuggieCli,
        "Augment Auggie",
        &["auggie"],
        AcpLaunch::executable_with_env(&["--acp"], &[("AUGMENT_DISABLE_AUTO_UPDATE", "1")]),
    ),
    agent(
        ExternalAgentKind::Cline,
        "Cline",
        &["cline"],
        AcpLaunch::executable(&["--acp"]),
    ),
    agent(
        ExternalAgentKind::Cursor,
        "Cursor",
        &["cursor-agent"],
        AcpLaunch::executable(&["acp"]),
    ),
    RegisteredAgent {
        desktop_client: Some(DesktopClient {
            #[cfg(target_os = "macos")]
            bundle_identifiers: &["com.google.antigravity", "com.google.antigravity-ide"],
            #[cfg(windows)]
            display_names: &["Antigravity", "Antigravity IDE"],
        }),
        ..agent(
            ExternalAgentKind::Antigravity,
            "Google Antigravity",
            &["agy_acp_server.par", "agy_acp_server"],
            AcpLaunch::Server,
        )
    },
    agent(
        ExternalAgentKind::GrokBuild,
        "xAI Grok Build",
        &["grok"],
        AcpLaunch::executable(&["agent", "stdio"]),
    ),
    agent(
        ExternalAgentKind::Goose,
        "Block Goose",
        &["goose"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::OpenInterpreter,
        "Open Interpreter",
        &["interpreter"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::Trae,
        "Trae",
        &["traecli"],
        AcpLaunch::executable(&["acp", "serve"]),
    ),
    agent(
        ExternalAgentKind::OpenHands,
        "OpenHands",
        &["openhands"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::Junie,
        "JetBrains Junie",
        &["junie"],
        AcpLaunch::executable(&["--acp=true"]),
    ),
    agent(
        ExternalAgentKind::MistralVibe,
        "Mistral Vibe",
        &["vibe-acp"],
        AcpLaunch::executable(&[]),
    ),
    agent(
        ExternalAgentKind::Codebuddy,
        "Tencent Codebuddy",
        &["codebuddy", "codebuddy-code", "cbc"],
        AcpLaunch::executable(&["--acp"]),
    ),
    agent(
        ExternalAgentKind::MiniMaxCode,
        "MiniMax Code",
        &["mcode"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::PiAcp,
        "pi ACP",
        &["pi-acp"],
        AcpLaunch::executable(&[]),
    ),
    agent(
        ExternalAgentKind::Hermes,
        "Nous Hermes",
        &["hermes", "hermes-acp"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::OpenClaw,
        "OpenClaw",
        &["openclaw"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::Devin,
        "Devin",
        &["devin"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::GlmAgent,
        "GLM Agent",
        &["glm-acp-agent"],
        AcpLaunch::executable(&[]),
    ),
    agent(
        ExternalAgentKind::Amp,
        "Amp",
        &["amp-acp"],
        AcpLaunch::executable(&[]),
    ),
    agent(
        ExternalAgentKind::FactoryDroid,
        "Factory Droid",
        &["droid"],
        AcpLaunch::executable_with_env(
            &["exec", "--output-format", "acp-daemon"],
            &[
                ("DROID_DISABLE_AUTO_UPDATE", "true"),
                ("FACTORY_DROID_AUTO_UPDATE_ENABLED", "false"),
            ],
        ),
    ),
    agent(
        ExternalAgentKind::Poolside,
        "Poolside",
        &["pool"],
        AcpLaunch::executable(&["acp"]),
    ),
    agent(
        ExternalAgentKind::Bob,
        "IBM Bob",
        &["bob"],
        AcpLaunch::executable(&["acp"]),
    ),
];
