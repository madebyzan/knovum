use std::{path::PathBuf, sync::Arc};

use agent_client_protocol::{
    schema::v1::{
        CancelNotification, McpServer, NewSessionRequest, PromptRequest, RequestPermissionRequest,
        StopReason,
    },
    Agent, Client, ConnectTo, ConnectionTo,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::ai::contracts::ExternalAgentKind;
use crate::app_error::{AppError, AppResult};

use super::session::AgentUpdateHandler;
mod requests;
mod start;

use requests::ConversationRequest;
pub use requests::{close, finish, is_active, send};
pub use start::start;

pub struct ConversationSession {
    pub runtime_session_id: String,
    pub capabilities: serde_json::Value,
    pub completion: oneshot::Receiver<AppResult<()>>,
}

struct ServeContext {
    kind: ExternalAgentKind,
    workspace: PathBuf,
    on_update: AgentUpdateHandler,
    mcp_servers: Vec<McpServer>,
    keepalive: Arc<dyn Send + Sync>,
    completion: oneshot::Receiver<AppResult<()>>,
}

async fn serve(
    context: ServeContext,
    requests: mpsc::Receiver<ConversationRequest>,
    cancellation: CancellationToken,
    ready: oneshot::Sender<AppResult<ConversationSession>>,
) -> AppResult<()> {
    let agent = super::connection::runtime(context.kind, &cancellation).await?;
    serve_connection(agent, context, requests, cancellation, ready).await
}

async fn serve_connection(
    agent: impl ConnectTo<Client> + 'static,
    context: ServeContext,
    mut requests: mpsc::Receiver<ConversationRequest>,
    cancellation: CancellationToken,
    ready: oneshot::Sender<AppResult<ConversationSession>>,
) -> AppResult<()> {
    let ServeContext {
        kind,
        workspace,
        on_update,
        mcp_servers,
        keepalive: _keepalive,
        completion,
    } = context;
    let permission_cancel = cancellation.clone();
    let client = Client
        .builder()
        .on_receive_request(
            async move |request: RequestPermissionRequest, responder, _cx| {
                responder.respond_with_result(super::permissions::authorize(
                    &request,
                    &permission_cancel,
                ))
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_close(async |_connection| {
            Err(agent_client_protocol::Error::internal_error()
                .data("External Agent connection closed"))
        });
    let session_cancel = cancellation.clone();
    let outcome = client.connect_with(agent, |connection: ConnectionTo<Agent>| async move {
        let initialized = connection.send_request(super::protocol::initialize_request())
            .block_task().await?;
        if !mcp_servers.is_empty() && !initialized.agent_capabilities.mcp_capabilities.http {
            let error = AppError::invalid("Desktop Agent does not support HTTP MCP servers");
            let _ = ready.send(Err(error));
            return Ok(false);
        }
        connection.build_session_from(NewSessionRequest::new(&workspace).mcp_servers(mcp_servers))
            .block_task().run_until(async move |mut session| {
                let permission_mode = async {
                    if let Some(request) = super::permissions::trusted_mode_request(
                        super::registry::trusted_session_mode(kind), &session.response(),
                    )? {
                        session.connection().send_request(request).block_task().await?;
                    }
                    Ok::<(), agent_client_protocol::Error>(())
                }.await;
                if let Err(error) = permission_mode {
                    let _ = ready.send(Err(AppError::invalid(error.to_string())));
                    return Err(error);
                }
                let session_capabilities = serde_json::to_value(session.response())
                    .map_err(|error| agent_client_protocol::Error::internal_error().data(error.to_string()))?;
                let _ = ready.send(Ok(ConversationSession {
                    runtime_session_id: session.session_id().0.to_string(),
                    capabilities: session_capabilities,
                    completion,
                }));
                loop {
                    let request = tokio::select! {
                        _ = session_cancel.cancelled() => return Ok(true),
                        request = requests.recv() => match request {
                            Some(ConversationRequest::Turn(value)) => value,
                            Some(ConversationRequest::Finish) | None => return Ok(false),
                        },
                    };
                    let content = request.prompt.into_content().await
                        .map_err(|error| agent_client_protocol::Error::invalid_params().data(error.to_string()))?;
                    let prompt = session.connection().send_request(
                        PromptRequest::new(session.session_id().clone(), content),
                    ).block_task();
                    tokio::pin!(prompt);
                    let turn_outcome = loop {
                        tokio::select! {
                            biased;
                            _ = session_cancel.cancelled() => {
                                session.connection().send_notification(
                                    CancelNotification::new(session.session_id().clone()),
                                )?;
                                break Err(AppError::cancelled("External Agent conversation was cancelled"));
                            }
                            message = session.read_update() => {
                                super::updates::receive_message(message?, &workspace, &on_update).await?;
                            }
                            response = &mut prompt => {
                                let response = response?;
                                break if response.stop_reason == StopReason::Cancelled {
                                    Err(AppError::cancelled("External Agent turn was cancelled"))
                                } else { Ok(()) };
                            }
                        }
                    };
                    let _ = request.reply.send(turn_outcome);
                }
            }).await
    }).await;
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled(
            "External Agent conversation was cancelled",
        ));
    }
    super::connection::finish(outcome)
}

#[cfg(test)]
mod tests;
