use agent_client_protocol::{AcpAgent, AcpAgentConfig};

use crate::ai::{
    contracts::{AgentDescriptor, AgentKind},
    error::{AiError, AiErrorCode, AiResult},
};

use super::{
    catalog::{AcpLaunch, RegisteredAgent, AGENTS},
    discovery,
};

fn registered(kind: AgentKind) -> &'static RegisteredAgent {
    AGENTS
        .iter()
        .find(|agent| agent.descriptor.kind == kind)
        .expect("every AgentKind must have one registry entry")
}

pub fn descriptors() -> Vec<AgentDescriptor> {
    AGENTS
        .iter()
        .map(|agent| agent.descriptor.clone())
        .collect()
}

pub(super) fn runtime(kind: AgentKind, path: &str) -> AiResult<AcpAgent> {
    let agent = registered(kind);
    let executable = agent
        .descriptor
        .executable_names
        .iter()
        .find_map(|name| discovery::find_executable(name, path))
        .ok_or_else(|| {
            AiError::new(
                AiErrorCode::ProviderRequestFailed,
                format!("{} execution entry was not found in PATH ({}). A desktop application installation does not provide this entry automatically.",
                    agent.descriptor.display_name, agent.descriptor.executable_names.join(", ")),
            )
        })?;
    let config = match agent.launch {
        AcpLaunch::Codex => configured_adapter(
            AcpAgent::codex().into_config(),
            path,
            "CODEX_PATH",
            &executable,
        ),
        AcpLaunch::Claude => configured_adapter(
            AcpAgent::claude_agent().into_config(),
            path,
            "CLAUDE_CODE_EXECUTABLE",
            &executable,
        ),
        AcpLaunch::Native { arguments } => AcpAgentConfig::new(executable)
            .args(arguments.iter().copied())
            .env("PATH", path),
        AcpLaunch::Server => AcpAgentConfig::new(executable).env("PATH", path),
    };
    Ok(AcpAgent::new(config))
}

fn configured_adapter(
    config: AcpAgentConfig,
    path: &str,
    executable_variable: &str,
    executable: &std::path::Path,
) -> AcpAgentConfig {
    config.env("PATH", path).env(
        executable_variable,
        executable.to_string_lossy().into_owned(),
    )
}
