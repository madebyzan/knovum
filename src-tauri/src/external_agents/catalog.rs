use crate::ai::contracts::{AgentConnectionKind, AgentDescriptor, AgentKind};

#[derive(Clone, Copy)]
pub(super) enum AcpLaunch {
    Codex,
    Claude,
    Native { arguments: &'static [&'static str] },
    Server,
}

impl AcpLaunch {
    pub const fn trusted_session_mode(self) -> Option<&'static str> {
        match self {
            Self::Codex => Some("full-access"),
            Self::Claude => Some("bypassPermissions"),
            Self::Native { .. } | Self::Server => None,
        }
    }

    const fn connection(self) -> AgentConnectionKind {
        match self {
            Self::Codex | Self::Claude => AgentConnectionKind::NodeAdapter,
            Self::Native { .. } => AgentConnectionKind::NativeCli,
            Self::Server => AgentConnectionKind::StandaloneServer,
        }
    }
}

pub(super) struct RegisteredAgent {
    pub descriptor: AgentDescriptor,
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
    kind: AgentKind,
    display_name: &'static str,
    executable_names: &'static [&'static str],
    launch: AcpLaunch,
) -> RegisteredAgent {
    RegisteredAgent {
        descriptor: AgentDescriptor {
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
        AgentKind::Codex,
        "OpenAI Codex",
        &["codex"],
        AcpLaunch::Codex,
    ),
    agent(
        AgentKind::ClaudeCode,
        "Anthropic Claude Code",
        &["claude"],
        AcpLaunch::Claude,
    ),
    agent(
        AgentKind::GeminiCli,
        "Google Gemini CLI",
        &["gemini"],
        AcpLaunch::Native {
            arguments: &["--acp"],
        },
    ),
    agent(
        AgentKind::GithubCopilot,
        "GitHub Copilot",
        &["copilot"],
        AcpLaunch::Native {
            arguments: &["--acp"],
        },
    ),
    agent(
        AgentKind::OpenCode,
        "OpenCode",
        &["opencode"],
        AcpLaunch::Native {
            arguments: &["acp"],
        },
    ),
    agent(
        AgentKind::QwenCode,
        "Qwen Code",
        &["qwen"],
        AcpLaunch::Native {
            arguments: &["--acp", "--experimental-skills"],
        },
    ),
    agent(
        AgentKind::QoderCli,
        "Qoder CLI",
        &["qodercli"],
        AcpLaunch::Native {
            arguments: &["--acp"],
        },
    ),
    agent(
        AgentKind::KimiCli,
        "Kimi CLI",
        &["kimi"],
        AcpLaunch::Native {
            arguments: &["acp"],
        },
    ),
    agent(
        AgentKind::AuggieCli,
        "Augment Auggie",
        &["auggie"],
        AcpLaunch::Native {
            arguments: &["--acp"],
        },
    ),
    agent(
        AgentKind::Cline,
        "Cline",
        &["cline"],
        AcpLaunch::Native {
            arguments: &["--acp"],
        },
    ),
    agent(
        AgentKind::Cursor,
        "Cursor",
        &["cursor-agent"],
        AcpLaunch::Native {
            arguments: &["acp"],
        },
    ),
    RegisteredAgent {
        desktop_client: Some(DesktopClient {
            #[cfg(target_os = "macos")]
            bundle_identifiers: &["com.google.antigravity", "com.google.antigravity-ide"],
            #[cfg(windows)]
            display_names: &["Antigravity", "Antigravity IDE"],
        }),
        ..agent(
            AgentKind::Antigravity,
            "Google Antigravity",
            &["agy_acp_server.par", "agy_acp_server"],
            AcpLaunch::Server,
        )
    },
];
