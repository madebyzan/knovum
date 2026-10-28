use std::sync::Arc;

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use agent_client_protocol::schema::v1::{HttpHeader, McpServer, McpServerHttp};

use super::agent_turn::{AgentTurn, AgentTurns};
use crate::extensions::contracts::AgentExtensionSnapshot;

/// The Knovum MCP server one desktop Agent runtime is given. It is announced to the Agent once,
/// when its session is created, and serves every turn of that runtime; each turn binds the
/// grant, Run and extensions it acts for.
pub(crate) struct AgentMcpBinding {
    endpoint: String,
    authorization: String,
    space_id: String,
    turns: AgentTurns,
    cancellation: CancellationToken,
}

impl AgentMcpBinding {
    pub(crate) fn server(&self) -> McpServer {
        McpServer::Http(
            McpServerHttp::new("knovum", self.endpoint.clone()).headers(vec![HttpHeader::new(
                "Authorization",
                self.authorization.clone(),
            )]),
        )
    }

    /// Authorizes `actor_id` — the Agent principal of this turn — and points every call the
    /// Agent makes until `end_turn` at `run_id`.
    pub(crate) async fn begin_turn(
        &self,
        app: &AppHandle,
        actor_id: &str,
        run_id: &str,
        extensions: Arc<AgentExtensionSnapshot>,
    ) -> Result<(), String> {
        let grant_id = crate::spaces::authorization::create_external_agent_grant(
            app,
            &self.space_id,
            actor_id,
            super::action_tools::external_agent_action_ids(),
        )
        .await?;
        self.turns.begin(AgentTurn {
            grant_id,
            run_id: run_id.to_owned(),
            extensions,
        })
    }

    pub(crate) fn end_turn(&self) {
        self.turns.end();
    }
}

impl Drop for AgentMcpBinding {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

/// Starts the server for a runtime whose session offers `extensions`.
pub(crate) async fn start(
    app: AppHandle,
    space_id: String,
    extensions: Arc<AgentExtensionSnapshot>,
) -> Result<AgentMcpBinding, String> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    let token = uuid::Uuid::new_v4().to_string();
    let cancellation = CancellationToken::new();
    let turns = AgentTurns::new(extensions);
    let router = super::transport::router(
        app,
        token.clone(),
        Some(space_id.clone()),
        turns.clone(),
        cancellation.child_token(),
    );
    let shutdown = cancellation.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = axum::serve(listener, router)
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
        {
            log::error!("Agent knowledge MCP server failed: {error}");
        }
    });
    Ok(AgentMcpBinding {
        endpoint: super::transport::endpoint(port),
        authorization: format!("Bearer {token}"),
        space_id,
        turns,
        cancellation,
    })
}
