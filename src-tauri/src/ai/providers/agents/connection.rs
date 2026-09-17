use agent_client_protocol::AcpAgent;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::AgentKind,
    error::{AiError, AiResult},
};

pub(super) const INITIALIZE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub(super) const START_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub(super) const SESSION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub(super) const CANCEL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) async fn runtime(
    kind: AgentKind,
    cancellation: &CancellationToken,
) -> AiResult<AcpAgent> {
    let path = tokio::select! {
        biased;
        _ = cancellation.cancelled() => {
            return Err(AiError::cancelled("Agent session was cancelled"));
        }
        path = super::discovery::system_path() => path,
    }?;
    super::registry::runtime(kind, &path)
}

pub(super) fn finish(outcome: Result<bool, agent_client_protocol::Error>) -> AiResult<()> {
    match outcome {
        Ok(true) => Err(AiError::cancelled("Agent session was cancelled")),
        Ok(false) => Ok(()),
        Err(error) => Err(error.into()),
    }
}
