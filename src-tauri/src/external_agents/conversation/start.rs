use std::{path::PathBuf, sync::Arc};

use agent_client_protocol::schema::v1::McpServer;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use super::{requests, serve, ConversationSession, ServeContext};
use crate::ai::contracts::ExternalAgentKind;
use crate::foundation::app_error::{AppError, AppResult};

pub async fn start(
    runtime_id: String,
    kind: ExternalAgentKind,
    workspace: PathBuf,
    mcp_servers: Vec<McpServer>,
    keepalive: Arc<dyn Send + Sync>,
) -> AppResult<ConversationSession> {
    let cancellation = CancellationToken::new();
    let receiver = requests::register(&runtime_id, cancellation.clone())?;
    let (ready_tx, ready_rx) = oneshot::channel();
    let (settled, completion) = oneshot::channel();
    // A runtime that fails before it has a session drops `ready` unanswered; its own error is
    // then the only account of why — the Agent was not found, refused to start or to sign in.
    let (failed_tx, failed_rx) = oneshot::channel();
    let registered = runtime_id.clone();
    tauri::async_runtime::spawn(async move {
        let context = ServeContext {
            kind,
            workspace,
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
            let _ = failed_tx.send(error.clone());
        }
        requests::remove(&registered);
        let _ = settled.send(outcome);
    });
    let created = tokio::time::timeout(super::super::connection::START_TIMEOUT, async {
        match ready_rx.await {
            Ok(created) => created,
            Err(_) => Err(failed_rx.await.unwrap_or_else(|_| {
                AppError::internal("ACP conversation ended before session creation")
            })),
        }
    })
    .await;
    match created {
        Ok(Ok(session)) => Ok(session),
        Ok(Err(error)) => {
            requests::remove(&runtime_id);
            Err(error)
        }
        Err(_) => {
            requests::close(&runtime_id)
                .await
                .map_err(AppError::internal)?;
            Err(AppError::internal(
                "ACP conversation timed out during session creation",
            ))
        }
    }
}
