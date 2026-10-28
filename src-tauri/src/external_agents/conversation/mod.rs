use std::{path::PathBuf, sync::Arc};

use agent_client_protocol::{
    schema::v1::{
        CancelNotification, McpServer, NewSessionRequest, PromptRequest, RequestPermissionRequest,
        StopReason,
    },
    ActiveSession, Agent, Client, ConnectTo, ConnectionTo,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::ai::contracts::ExternalAgentKind;
use crate::foundation::app_error::{AppError, AppResult};

mod requests;
mod start;

use requests::TurnRequest;
pub use requests::{close, is_active, send};
pub use start::start;

/// One Agent process and the one ACP session it serves. It lives across turns: the Agent keeps
/// what was said and done, so a follow-up is sent as it is, without the conversation replayed.
pub struct ConversationSession {
    pub runtime_session_id: String,
    pub capabilities: serde_json::Value,
    /// Settles when the runtime ends — closed, cancelled, or lost.
    pub completion: oneshot::Receiver<AppResult<()>>,
}

struct ServeContext {
    kind: ExternalAgentKind,
    workspace: PathBuf,
    mcp_servers: Vec<McpServer>,
    keepalive: Arc<dyn Send + Sync>,
    completion: oneshot::Receiver<AppResult<()>>,
}

async fn serve(
    context: ServeContext,
    requests: mpsc::Receiver<TurnRequest>,
    cancellation: CancellationToken,
    ready: oneshot::Sender<AppResult<ConversationSession>>,
) -> AppResult<()> {
    let agent =
        super::connection::runtime(context.kind, &context.mcp_servers, &cancellation).await?;
    serve_connection(agent, context, requests, cancellation, ready).await
}

async fn serve_connection(
    agent: impl ConnectTo<Client> + 'static,
    context: ServeContext,
    mut requests: mpsc::Receiver<TurnRequest>,
    cancellation: CancellationToken,
    ready: oneshot::Sender<AppResult<ConversationSession>>,
) -> AppResult<()> {
    let ServeContext {
        kind,
        workspace,
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
    let outcome = client
        .connect_with(agent, |connection: ConnectionTo<Agent>| async move {
            let initialized = connection
                .send_request(super::protocol::initialize_request())
                .block_task()
                .await?;
            if !mcp_servers.is_empty() && !initialized.agent_capabilities.mcp_capabilities.http {
                let error = AppError::invalid("Desktop Agent does not support HTTP MCP servers");
                let _ = ready.send(Err(error));
                return Ok(false);
            }
            connection
                .build_session_from(NewSessionRequest::new(&workspace).mcp_servers(mcp_servers))
                .block_task()
                .run_until(async move |mut session| {
                    let permission_mode = async {
                        if let Some(request) = super::permissions::trusted_mode_request(
                            super::registry::trusted_session_mode(kind),
                            &session.response(),
                        )? {
                            session
                                .connection()
                                .send_request(request)
                                .block_task()
                                .await?;
                        }
                        Ok::<(), agent_client_protocol::Error>(())
                    }
                    .await;
                    if let Err(error) = permission_mode {
                        let _ = ready.send(Err(AppError::invalid(error.to_string())));
                        return Err(error);
                    }
                    let session_capabilities =
                        serde_json::to_value(session.response()).map_err(|error| {
                            agent_client_protocol::Error::internal_error().data(error.to_string())
                        })?;
                    let _ = ready.send(Ok(ConversationSession {
                        runtime_session_id: session.session_id().0.to_string(),
                        capabilities: session_capabilities,
                        completion,
                    }));
                    loop {
                        let request = tokio::select! {
                            _ = session_cancel.cancelled() => return Ok(true),
                            request = requests.recv() => match request {
                                Some(request) => request,
                                None => return Ok(false),
                            },
                        };
                        if !run_turn(&mut session, request, &workspace, &session_cancel).await? {
                            return Ok(true);
                        }
                    }
                })
                .await
        })
        .await;
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled(
            "External Agent conversation was cancelled",
        ));
    }
    super::connection::finish(outcome)
}

/// Runs one prompt to its end. A turn that is cancelled, or whose updates cannot be kept, asks
/// the Agent to stop and still waits for its answer to the prompt, so the session is left
/// between turns rather than mid-turn; only a lost connection, a runtime being closed, or an
/// Agent that ignores the cancellation ends the runtime. Returns whether the runtime goes on.
async fn run_turn(
    session: &mut ActiveSession<'_, Agent>,
    request: TurnRequest,
    workspace: &std::path::Path,
    runtime: &CancellationToken,
) -> Result<bool, agent_client_protocol::Error> {
    let TurnRequest {
        prompt,
        on_update,
        cancellation,
        reply,
    } = request;
    let content = match prompt.into_content().await {
        Ok(content) => content,
        Err(error) => {
            let _ = reply.send(Err(error));
            return Ok(true);
        }
    };
    let prompt = session
        .connection()
        .send_request(PromptRequest::new(session.session_id().clone(), content))
        .block_task();
    tokio::pin!(prompt);
    let stop_deadline = tokio::time::sleep(std::time::Duration::MAX);
    tokio::pin!(stop_deadline);
    let mut failure: Option<AppError> = None;
    let mut stopping = false;
    loop {
        let stop = tokio::select! {
            biased;
            _ = runtime.cancelled() => {
                let _ = reply.send(Err(AppError::cancelled("External Agent conversation was closed")));
                return Ok(false);
            }
            _ = &mut stop_deadline, if stopping => {
                let _ = reply.send(Err(failure.unwrap_or_else(|| AppError::cancelled("External Agent turn was cancelled"))));
                return Err(agent_client_protocol::Error::internal_error()
                    .data("External Agent did not stop its turn when asked"));
            }
            _ = cancellation.cancelled(), if !stopping => true,
            message = session.read_update() => match message {
                Ok(message) => match super::updates::receive_message(message, workspace, &on_update).await {
                    Ok(()) => false,
                    Err(error) => {
                        failure.get_or_insert_with(|| error.into());
                        !stopping
                    }
                },
                Err(error) => {
                    let _ = reply.send(Err(error.clone().into()));
                    return Err(error);
                }
            },
            response = &mut prompt => {
                let outcome = match (failure, response) {
                    (Some(error), _) => Err(error),
                    (None, Err(error)) => Err(error.into()),
                    (None, Ok(response)) if response.stop_reason == StopReason::Cancelled => {
                        Err(AppError::cancelled("External Agent turn was cancelled"))
                    }
                    (None, Ok(_)) => Ok(()),
                };
                let _ = reply.send(outcome);
                return Ok(true);
            }
        };
        if stop && !stopping {
            stopping = true;
            session
                .connection()
                .send_notification(CancelNotification::new(session.session_id().clone()))?;
            stop_deadline
                .as_mut()
                .reset(tokio::time::Instant::now() + super::connection::CANCEL_TIMEOUT);
        }
    }
}

#[cfg(test)]
mod tests;
