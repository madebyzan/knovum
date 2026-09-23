use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use agent_client_protocol::schema::v1::{HttpHeader, McpServer, McpServerHttp};

pub(crate) struct AgentMcpBinding {
    endpoint: String,
    authorization: String,
    cancellation: CancellationToken,
}

impl AgentMcpBinding {
    pub(crate) fn server(&self) -> McpServer {
        McpServer::Http(
            McpServerHttp::new("Knovum Knowledge", self.endpoint.clone()).headers(vec![
                HttpHeader::new("Authorization", self.authorization.clone()),
            ]),
        )
    }
}

impl Drop for AgentMcpBinding {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

pub(crate) async fn start(
    app: AppHandle,
    space_id: String,
    actor_id: String,
) -> Result<AgentMcpBinding, String> {
    let grant_id = create_session_grant(&app, &space_id, &actor_id).await?;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    let token = uuid::Uuid::new_v4().to_string();
    let cancellation = CancellationToken::new();
    let router = super::transport::router(
        app,
        token.clone(),
        Some(space_id),
        grant_id,
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
        cancellation,
    })
}

async fn create_session_grant(
    app: &AppHandle,
    space_id: &str,
    actor_id: &str,
) -> Result<String, String> {
    crate::spaces::authorization::create_external_agent_grant(app, space_id, actor_id).await
}
