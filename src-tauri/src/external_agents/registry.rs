use agent_client_protocol::{AcpAgent, AcpAgentConfig};

use crate::ai::contracts::{ExternalAgentDescriptor, ExternalAgentKind};
use crate::foundation::app_error::{AppError, AppResult};

use super::{
    catalog::{AcpLaunch, RegisteredAgent, AGENTS},
    discovery,
};

fn registered(kind: ExternalAgentKind) -> &'static RegisteredAgent {
    AGENTS
        .iter()
        .find(|agent| agent.descriptor.kind == kind)
        .expect("every ExternalAgentKind must have one registry entry")
}

pub(super) fn trusted_session_mode(kind: ExternalAgentKind) -> Option<&'static str> {
    registered(kind).launch.trusted_session_mode()
}

pub fn descriptors() -> Vec<ExternalAgentDescriptor> {
    AGENTS
        .iter()
        .map(|agent| agent.descriptor.clone())
        .collect()
}

pub(super) fn runtime(
    kind: ExternalAgentKind,
    path: &str,
    environment: Vec<(&'static str, String)>,
) -> AppResult<AcpAgent> {
    let agent = registered(kind);
    let executable = agent
        .descriptor
        .executable_names
        .iter()
        .find_map(|name| discovery::find_executable(name, path))
        .ok_or_else(|| {
            AppError::new(
                "agent_unavailable",
                format!("{} execution entry was not found in PATH ({}). A desktop application installation does not provide this entry automatically.",
                    agent.descriptor.display_name, agent.descriptor.executable_names.join(", ")),
            )
        })?;
    let config = match agent.launch {
        AcpLaunch::Codex => AcpAgent::codex()
            .into_config()
            .env("CODEX_PATH", executable.to_string_lossy()),
        AcpLaunch::Claude => AcpAgent::claude_agent()
            .into_config()
            .env("CLAUDE_CODE_EXECUTABLE", executable.to_string_lossy()),
        AcpLaunch::Executable { arguments } => {
            AcpAgentConfig::new(executable).args(arguments.iter().copied())
        }
        AcpLaunch::Server => AcpAgentConfig::new(executable),
    };
    Ok(AcpAgent::new(config.env("PATH", path).envs(environment)))
}
