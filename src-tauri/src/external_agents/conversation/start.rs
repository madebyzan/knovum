use std::{path::PathBuf, sync::Arc};

use agent_client_protocol::schema::v1::McpServer;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use super::{requests, serve, ConversationSession, ServeContext};
use crate::{
    ai::{
        contracts::AgentKind,
        error::{AiError, AiResult},
    },
    external_agents::session::AgentUpdateHandler,
};

pub async fn start(
    id: String,
    kind: AgentKind,
    workspace: PathBuf,
    on_update: AgentUpdateHandler,
    mcp_servers: Vec<McpServer>,
    keepalive: Arc<dyn Send + Sync>,
) -> AiResult<ConversationSession> {
    let cancellation = CancellationToken::new();
    let receiver = requests::register(&id, cancellation.clone())?;
    let (ready_tx, ready_rx) = oneshot::channel();
    let runtime_id = id.clone();
    tauri::async_runtime::spawn(async move {
        let context = ServeContext {
            id: runtime_id.clone(),
            kind,
            workspace,
            on_update,
            mcp_servers,
            keepalive,
        };
        if let Err(error) = serve(context, receiver, cancellation, ready_tx).await {
            log::error!("External Agent conversation ended: {error}");
        }
        requests::remove(&runtime_id);
    });
    match tokio::time::timeout(super::super::connection::START_TIMEOUT, ready_rx).await {
        Ok(Ok(Ok(session))) => Ok(session),
        Ok(Ok(Err(error))) => {
            requests::remove(&id);
            Err(error)
        }
        Ok(Err(_)) => {
            requests::remove(&id);
            Err(AiError::internal(
                "ACP conversation ended before session creation",
            ))
        }
        Err(_) => {
            requests::close(&id).await.map_err(AiError::internal)?;
            Err(AiError::internal(
                "ACP conversation timed out during session creation",
            ))
        }
    }
}
