use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    Router,
};
use ring::hmac;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::SpaceMcpServer;

const MCP_HOST: &str = "127.0.0.1";
const MCP_PATH: &str = "/mcp";
const BEARER_COMPARISON_KEY: &[u8] = b"knovum-spaces-mcp-bearer-credential";

pub(super) fn router(
    app: AppHandle,
    token: String,
    space_scope: Option<String>,
    turns: super::agent_turn::AgentTurns,
    cancellation: CancellationToken,
) -> Router {
    let mcp_transport: StreamableHttpService<SpaceMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || {
                Ok(SpaceMcpServer {
                    app: app.clone(),
                    space_scope: space_scope.clone(),
                    turns: turns.clone(),
                })
            },
            Default::default(),
            StreamableHttpServerConfig::default().with_cancellation_token(cancellation),
        );
    Router::new()
        .nest_service(MCP_PATH, mcp_transport)
        .layer(middleware::from_fn_with_state(token, require_bearer))
}

pub(super) fn endpoint(port: u16) -> String {
    format!("http://{MCP_HOST}:{port}{MCP_PATH}")
}

async fn require_bearer(
    State(token): State<String>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected = format!("Bearer {token}");
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| bearer_matches(&expected, value));
    if !authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}

/// `ring::hmac::verify` recomputes the tag for the presented credential and compares it
/// with `CRYPTO_memcmp`, so the bearer token is never compared byte by byte.
fn bearer_matches(expected: &str, presented: &str) -> bool {
    let key = hmac::Key::new(hmac::HMAC_SHA256, BEARER_COMPARISON_KEY);
    let tag = hmac::sign(&key, expected.as_bytes());
    hmac::verify(&key, presented.as_bytes(), tag.as_ref()).is_ok()
}
