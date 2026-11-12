use agent_client_protocol::{AcpAgent, AcpAgentConfig};

use crate::ai::contracts::{ExternalAgentDescriptor, ExternalAgentKind};
use crate::foundation::app_error::{AppError, AppResult};

use super::{
    catalog::{AcpLaunch, RegisteredAgent, AGENTS, CLAUDE_ADAPTER, CODEX_ADAPTER},
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
    let mut config = match agent.launch {
        AcpLaunch::Codex => {
            node_adapter(CODEX_ADAPTER, path)?.env("CODEX_PATH", executable.to_string_lossy())
        }
        AcpLaunch::Claude => node_adapter(CLAUDE_ADAPTER, path)?
            .env("CLAUDE_CODE_EXECUTABLE", executable.to_string_lossy()),
        AcpLaunch::Executable { arguments, env } => {
            let mut launched = AcpAgentConfig::new(executable).args(arguments.iter().copied());
            for (name, value) in env {
                launched = launched.env(*name, *value);
            }
            launched
        }
        AcpLaunch::Server => AcpAgentConfig::new(executable),
    };
    config = config.env("PATH", path).envs(environment);
    Ok(AcpAgent::new(config))
}

fn node_adapter(package: &str, path: &str) -> AppResult<AcpAgentConfig> {
    let npx = discovery::find_executable("npx", path).ok_or_else(|| {
        AppError::new(
            "agent_unavailable",
            "Node.js package runner (npx) was not found in PATH. Install Node.js to use this Agent.",
        )
    })?;
    Ok(AcpAgentConfig::new(npx).args(["-y", package]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_adapter_fails_when_npx_is_missing() {
        let err = node_adapter("any-package", "").expect_err("should fail when npx not found");
        assert_eq!(err.code, "agent_unavailable");
        assert!(err
            .message
            .contains("Node.js package runner (npx) was not found in PATH"));
    }
}
