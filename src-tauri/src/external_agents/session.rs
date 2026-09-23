use std::{path::PathBuf, sync::Arc};

use agent_client_protocol::{
    schema::v1::{
        CancelNotification, McpServer, NewSessionRequest, PromptRequest, RequestPermissionRequest,
        StopReason,
    },
    Agent, Client, ConnectionTo,
};
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::AgentKind,
    error::{AiError, AiResult},
};

pub use super::updates::AgentSessionUpdate;

pub type AgentUpdateHandler = Arc<dyn Fn(AgentSessionUpdate) -> AiResult<()> + Send + Sync>;

pub async fn run(
    task_id: String,
    kind: AgentKind,
    working_directory: PathBuf,
    prompt: super::prompt::AgentPrompt,
    cancellation: CancellationToken,
    on_update: AgentUpdateHandler,
    mcp_servers: Vec<McpServer>,
) -> AiResult<()> {
    let agent = super::connection::runtime(kind, &cancellation).await?;
    let permission_cancellation = cancellation.clone();
    let permission_updates = on_update.clone();
    let client = Client.builder().on_receive_request(
        async move |request: RequestPermissionRequest, responder, _cx| {
            let response = super::permissions::request_decision(
                &task_id,
                &request,
                &permission_cancellation,
                &permission_updates,
            )
            .await;
            responder.respond(response)
        },
        agent_client_protocol::on_receive_request!(),
    );

    let session_cancellation = cancellation.clone();
    let (connection_started, mut connection_start) = tokio::sync::oneshot::channel();
    let connection = client.connect_with(agent, |connection: ConnectionTo<Agent>| async move {
        let _ = connection_started.send(());
        let initialize = connection
            .send_request(super::protocol::initialize_request())
            .block_task();
        let initialization = tokio::select! {
            biased;
            _ = session_cancellation.cancelled() => {
                return Err(agent_client_protocol::Error::request_cancelled());
            }
            result = tokio::time::timeout(super::connection::INITIALIZE_TIMEOUT, initialize) => {
                result
                    .map_err(|_| agent_client_protocol::Error::internal_error().data(
                        "ACP initialization timed out after 30 seconds",
                    ))??
            }
        };
        if !mcp_servers.is_empty() && !initialization.agent_capabilities.mcp_capabilities.http {
            return Err(agent_client_protocol::Error::invalid_request().data(
                "Desktop Agent does not support HTTP MCP servers",
            ));
        }
        let prepare_content = prompt.into_content(
            &working_directory,
            initialization.agent_capabilities.prompt_capabilities,
        );
        let content = tokio::select! {
            biased;
            _ = session_cancellation.cancelled() => {
                return Err(agent_client_protocol::Error::request_cancelled());
            }
            result = prepare_content => result.map_err(|error| {
                agent_client_protocol::Error::invalid_params().data(error.to_string())
            })?,
        };
        let (session_started, mut session_start) = tokio::sync::oneshot::channel();
        let prompt_cancellation = session_cancellation.clone();
        let session_run = connection
            .build_session_from(NewSessionRequest::new(&working_directory).mcp_servers(mcp_servers))
            .block_task()
            .run_until(async move |mut session| {
                let _ = session_started.send(());
                let session_id = session.session_id().clone();
                let prompt = session
                    .connection()
                    .send_request(PromptRequest::new(session_id.clone(), content))
                    .block_task();
                tokio::pin!(prompt);
                loop {
                    tokio::select! {
                        biased;
                        _ = prompt_cancellation.cancelled() => {
                            session.connection().send_notification(
                                CancelNotification::new(session_id),
                            )?;
                            let deadline = tokio::time::sleep(super::connection::CANCEL_TIMEOUT);
                            tokio::pin!(deadline);
                            loop {
                                tokio::select! {
                                    message = session.read_update() => {
                                        super::updates::receive_message(
                                            message?, &working_directory, &on_update,
                                        ).await?;
                                    }
                                    response = &mut prompt => {
                                        if response?.stop_reason != StopReason::Cancelled {
                                            return Err(agent_client_protocol::Error::internal_error().data(
                                                "Agent completed without acknowledging session cancellation",
                                            ));
                                        }
                                        return Ok(true);
                                    }
                                    _ = &mut deadline => {
                                        return Err(agent_client_protocol::Error::internal_error().data(
                                            "Agent did not acknowledge session cancellation",
                                        ));
                                    }
                                }
                            }
                        }
                        message = session.read_update() => {
                            super::updates::receive_message(
                                message?,
                                &working_directory,
                                &on_update,
                            ).await?;
                        }
                        response = &mut prompt => {
                            return Ok(response?.stop_reason == StopReason::Cancelled);
                        }
                    }
                }
            });
        tokio::pin!(session_run);
        tokio::select! {
            biased;
            result = &mut session_run => result,
            started = &mut session_start => {
                started.map_err(|_| agent_client_protocol::Error::internal_error().data(
                    "ACP session start ended without creating a session",
                ))?;
                session_run.await
            }
            _ = session_cancellation.cancelled() => {
                Err(agent_client_protocol::Error::request_cancelled())
            }
            _ = tokio::time::sleep(super::connection::SESSION_TIMEOUT) => {
                Err(agent_client_protocol::Error::internal_error().data(
                    "ACP session creation timed out after 30 seconds",
                ))
            }
        }
    });

    tokio::pin!(connection);
    let outcome = tokio::select! {
        biased;
        result = &mut connection => result,
        started = &mut connection_start => {
            started.map_err(|_| AiError::internal(
                "ACP connection ended before initialization",
            ))?;
            connection.await
        }
        _ = cancellation.cancelled() => {
            return Err(AiError::cancelled("Agent session was cancelled"));
        }
        _ = tokio::time::sleep(super::connection::START_TIMEOUT) => {
            return Err(AiError::internal("ACP connection timed out after 30 seconds"));
        }
    };
    super::connection::finish(outcome)
}
