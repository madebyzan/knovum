use crate::ai::contracts::{
    ExternalAgentConnectionKind, ExternalAgentDescriptor, ExternalAgentKind,
};

#[derive(Clone, Copy)]
pub(super) enum AcpLaunch {
    Codex,
    Claude,
    Executable { arguments: &'static [&'static str] },
    Server,
}

impl AcpLaunch {
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
        AcpLaunch::Executable {
            arguments: &["--acp"],
        },
    ),
    agent(
        ExternalAgentKind::GithubCopilot,
        "GitHub Copilot",
        &["copilot"],
        AcpLaunch::Executable {
            arguments: &["--acp"],
        },
    ),
    agent(
        ExternalAgentKind::OpenCode,
        "OpenCode",
        &["opencode"],
        AcpLaunch::Executable {
            arguments: &["acp"],
        },
    ),
    agent(
        ExternalAgentKind::QwenCode,
        "Qwen Code",
        &["qwen"],
        AcpLaunch::Executable {
            arguments: &["--acp", "--experimental-skills"],
        },
    ),
    agent(
        ExternalAgentKind::QoderCli,
        "Qoder CLI",
        &["qodercli"],
        AcpLaunch::Executable {
            arguments: &["--acp"],
        },
    ),
    agent(
        ExternalAgentKind::KimiCli,
        "Kimi CLI",
        &["kimi"],
        AcpLaunch::Executable {
            arguments: &["acp"],
        },
    ),
    agent(
        ExternalAgentKind::AuggieCli,
        "Augment Auggie",
        &["auggie"],
        AcpLaunch::Executable {
            arguments: &["--acp"],
        },
    ),
    agent(
        ExternalAgentKind::Cline,
        "Cline",
        &["cline"],
        AcpLaunch::Executable {
            arguments: &["--acp"],
        },
    ),
    agent(
        ExternalAgentKind::Cursor,
        "Cursor",
        &["cursor-agent"],
        AcpLaunch::Executable {
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
            ExternalAgentKind::Antigravity,
            "Google Antigravity",
            &["agy_acp_server.par", "agy_acp_server"],
            AcpLaunch::Server,
        )
    },
];
