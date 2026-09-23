use std::{path::PathBuf, sync::Arc};

use agent_client_protocol::{
    schema::v1::{
        CancelNotification, McpServer, NewSessionRequest, PromptRequest, RequestPermissionRequest,
        SessionConfigOptionValue, SetSessionConfigOptionRequest, SetSessionModeRequest, StopReason,
    },
    Agent, Client, ConnectionTo,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::AgentKind,
    error::{AiError, AiResult},
};

use super::session::AgentUpdateHandler;
mod requests;
mod start;

use requests::ConversationRequest;
pub use requests::{close, configure, is_active, send, set_mode};
pub use start::start;

pub struct ConversationSession {
    pub runtime_session_id: String,
    pub capabilities: serde_json::Value,
}

struct ServeContext {
    id: String,
    kind: AgentKind,
    workspace: PathBuf,
    on_update: AgentUpdateHandler,
    mcp_servers: Vec<McpServer>,
    keepalive: Arc<dyn Send + Sync>,
}

async fn serve(
    context: ServeContext,
    mut requests: mpsc::Receiver<ConversationRequest>,
    cancellation: CancellationToken,
    ready: oneshot::Sender<AiResult<ConversationSession>>,
) -> AiResult<()> {
    let ServeContext {
        id,
        kind,
        workspace,
        on_update,
        mcp_servers,
        keepalive: _keepalive,
    } = context;
    let agent = super::connection::runtime(kind, &cancellation).await?;
    let permission_cancel = cancellation.clone();
    let permission_updates = on_update.clone();
    let permission_id = id.clone();
    let client = Client.builder().on_receive_request(
        async move |request: RequestPermissionRequest, responder, _cx| {
            let response = super::permissions::request_decision(
                &permission_id,
                &request,
                &permission_cancel,
                &permission_updates,
            )
            .await;
            responder.respond(response)
        },
        agent_client_protocol::on_receive_request!(),
    );
    let session_cancel = cancellation.clone();
    let outcome = client.connect_with(agent, |connection: ConnectionTo<Agent>| async move {
        let initialized = connection.send_request(super::protocol::initialize_request())
            .block_task().await?;
        if !mcp_servers.is_empty() && !initialized.agent_capabilities.mcp_capabilities.http {
            let error = AiError::invalid("Desktop Agent does not support HTTP MCP servers");
            let _ = ready.send(Err(error));
            return Ok(false);
        }
        let prompt_capabilities = initialized.agent_capabilities.prompt_capabilities;
        connection.build_session_from(NewSessionRequest::new(&workspace).mcp_servers(mcp_servers))
            .block_task().run_until(async move |mut session| {
                let session_capabilities = serde_json::to_value(session.response())
                    .map_err(|error| agent_client_protocol::Error::internal_error().data(error.to_string()))?;
                let _ = ready.send(Ok(ConversationSession {
                    runtime_session_id: session.session_id().0.to_string(),
                    capabilities: session_capabilities,
                }));
                loop {
                    let request = tokio::select! {
                        _ = session_cancel.cancelled() => return Ok(true),
                        request = requests.recv() => match request { Some(value) => value, None => return Ok(false) },
                    };
                    let request = match request {
                        ConversationRequest::Turn(request) => request,
                        ConversationRequest::Configure(request) => {
                            let value = match request.value {
                                serde_json::Value::Bool(value) => SessionConfigOptionValue::boolean(value),
                                serde_json::Value::String(value) => SessionConfigOptionValue::value_id(value),
                                _ => {
                                    let _ = request.reply.send(Err(AiError::invalid(
                                        "ACP session option value must be a string or boolean",
                                    )));
                                    continue;
                                }
                            };
                            let response = session.connection().send_request(
                                SetSessionConfigOptionRequest::new(
                                    session.session_id().clone(), request.id, value,
                                ),
                            ).block_task().await;
                            let result = response.map_err(|error| AiError::internal(error.to_string()))
                                .and_then(|response| serde_json::to_value(response.config_options)
                                    .map_err(|error| AiError::internal(error.to_string())));
                            let _ = request.reply.send(result);
                            continue;
                        }
                        ConversationRequest::SetMode(request) => {
                            let result = session.connection().send_request(SetSessionModeRequest::new(
                                session.session_id().clone(), request.id,
                            )).block_task().await
                                .map(|_| ())
                                .map_err(|error| AiError::internal(error.to_string()));
                            let _ = request.reply.send(result);
                            continue;
                        }
                    };
                    let content = request.prompt.into_content(&workspace, prompt_capabilities.clone()).await
                        .map_err(|error| agent_client_protocol::Error::invalid_params().data(error.to_string()))?;
                    let prompt = session.connection().send_request(
                        PromptRequest::new(session.session_id().clone(), content),
                    ).block_task();
                    tokio::pin!(prompt);
                    let result = loop {
                        tokio::select! {
                            _ = session_cancel.cancelled() => {
                                session.connection().send_notification(
                                    CancelNotification::new(session.session_id().clone()),
                                )?;
                                break Err(AiError::cancelled("External Agent conversation was cancelled"));
                            }
                            message = session.read_update() => {
                                super::updates::receive_message(message?, &workspace, &on_update).await?;
                            }
                            response = &mut prompt => {
                                let response = response?;
                                break if response.stop_reason == StopReason::Cancelled {
                                    Err(AiError::cancelled("External Agent turn was cancelled"))
                                } else { Ok(()) };
                            }
                        }
                    };
                    let _ = request.reply.send(result);
                }
            }).await
    }).await;
    super::connection::finish(outcome)
}
