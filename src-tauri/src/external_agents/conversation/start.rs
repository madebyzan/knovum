use std::{path::PathBuf, sync::Arc};

use agent_client_protocol::schema::v1::McpServer;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use super::{requests, serve, ConversationSession, ServeContext};
use crate::app_error::{AppError, AppResult};
use crate::{ai::contracts::ExternalAgentKind, external_agents::session::AgentUpdateHandler};

pub async fn start(
    id: String,
    kind: ExternalAgentKind,
    workspace: PathBuf,
    on_update: AgentUpdateHandler,
    mcp_servers: Vec<McpServer>,
    keepalive: Arc<dyn Send + Sync>,
) -> AppResult<ConversationSession> {
    let cancellation = CancellationToken::new();
    let receiver = requests::register(&id, cancellation.clone())?;
    let (ready_tx, ready_rx) = oneshot::channel();
    let (settled, completion) = oneshot::channel();
    let runtime_id = id.clone();
    tauri::async_runtime::spawn(async move {
        let context = ServeContext {
            kind,
            workspace,
            on_update,
            mcp_servers,
            keepalive,
            completion,
        };
        let outcome = serve(context, receiver, cancellation, ready_tx).await;
        if let Err(error) = &outcome {
            if error.code == "cancelled" {
                log::debug!("External Agent conversation cancelled: {error}");
            } else {
                log::error!("External Agent conversation ended: {error}");
            }
        }
        requests::remove(&runtime_id);
        let _ = settled.send(outcome);
    });
    match tokio::time::timeout(super::super::connection::START_TIMEOUT, ready_rx).await {
        Ok(Ok(Ok(session))) => Ok(session),
        Ok(Ok(Err(error))) => {
            requests::remove(&id);
            Err(error)
        }
        Ok(Err(_)) => {
            requests::remove(&id);
            Err(AppError::internal(
                "ACP conversation ended before session creation",
            ))
        }
        Err(_) => {
            requests::close(&id).await.map_err(AppError::internal)?;
            Err(AppError::internal(
                "ACP conversation timed out during session creation",
            ))
        }
    }
}
