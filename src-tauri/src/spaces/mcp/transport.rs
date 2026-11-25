use std::sync::OnceLock;

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    Router,
};
use ring::{
    hmac,
    rand::{SecureRandom, SystemRandom},
};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::SpaceMcpServer;

const MCP_HOST: &str = "127.0.0.1";
const MCP_PATH: &str = "/mcp";
const MAX_MCP_BODY_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone)]
struct GuardState {
    expected_host: String,
    token: String,
}

fn comparison_key() -> &'static hmac::Key {
    static KEY: OnceLock<hmac::Key> = OnceLock::new();
    KEY.get_or_init(|| {
        let mut bytes = [0u8; 32];
        let rng = SystemRandom::new();
        if rng.fill(&mut bytes).is_err() {
            // Fallback to two UUIDv4 values if OS CSPRNG call ever fails unexpectedly.
            bytes[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
            bytes[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        }
        hmac::Key::new(hmac::HMAC_SHA256, &bytes)
    })
}

pub(super) fn router(
    app: AppHandle,
    port: u16,
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
    let guard = GuardState {
        expected_host: format!("{MCP_HOST}:{port}"),
        token,
    };
    Router::new()
        .nest_service(MCP_PATH, mcp_transport)
        .layer(middleware::from_fn_with_state(guard, guard_request))
}

pub(super) fn endpoint(port: u16) -> String {
    format!("http://{MCP_HOST}:{port}{MCP_PATH}")
}

async fn guard_request(
    State(guard): State<GuardState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.headers().contains_key(header::ORIGIN) {
        return Err(StatusCode::FORBIDDEN);
    }
    let host_ok = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host == guard.expected_host);
    if !host_ok {
        return Err(StatusCode::FORBIDDEN);
    }
    let expected = format!("Bearer {}", guard.token);
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| bearer_matches(&expected, value));
    if !authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if let Some(length) = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
    {
        if length > MAX_MCP_BODY_BYTES {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
    }
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, MAX_MCP_BODY_BYTES)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let bounded = Request::from_parts(parts, Body::from(bytes));
    Ok(next.run(bounded).await)
}

/// `ring::hmac::verify` recomputes the tag for the presented credential and compares it
/// with `CRYPTO_memcmp`, so the bearer token is never compared byte by byte.
fn bearer_matches(expected: &str, presented: &str) -> bool {
    let key = comparison_key();
    let tag = hmac::sign(key, expected.as_bytes());
    hmac::verify(key, presented.as_bytes(), tag.as_ref()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::post;

    async fn spawn_guarded_app(token: &str) -> (u16, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let guard = GuardState {
            expected_host: format!("{MCP_HOST}:{port}"),
            token: token.to_owned(),
        };
        let app = Router::new()
            .route(MCP_PATH, post(|| async { StatusCode::OK }))
            .layer(middleware::from_fn_with_state(guard, guard_request));
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        (port, handle)
    }

    #[tokio::test]
    async fn rejects_cross_origin_and_dns_rebinding_hosts_and_oversized_bodies() {
        let (port, handle) = spawn_guarded_app("secret-token").await;
        let url = endpoint(port);
        let client = reqwest::Client::builder().no_proxy().build().unwrap();

        let bad_host = client
            .post(&url)
            .header(header::HOST, format!("evil.example:{port}"))
            .header(header::AUTHORIZATION, "Bearer secret-token")
            .send()
            .await
            .unwrap();
        assert_eq!(bad_host.status(), reqwest::StatusCode::FORBIDDEN);

        let with_origin = client
            .post(&url)
            .header(header::ORIGIN, "https://attacker.example")
            .header(header::AUTHORIZATION, "Bearer secret-token")
            .send()
            .await
            .unwrap();
        assert_eq!(with_origin.status(), reqwest::StatusCode::FORBIDDEN);

        let bad_bearer = client
            .post(&url)
            .header(header::AUTHORIZATION, "Bearer wrong-token")
            .send()
            .await
            .unwrap();
        assert_eq!(bad_bearer.status(), reqwest::StatusCode::UNAUTHORIZED);

        let oversized = client
            .post(&url)
            .header(header::AUTHORIZATION, "Bearer secret-token")
            .body(vec![b'x'; MAX_MCP_BODY_BYTES + 1])
            .send()
            .await
            .unwrap();
        assert_eq!(oversized.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);

        let valid = client
            .post(&url)
            .header(header::AUTHORIZATION, "Bearer secret-token")
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(valid.status(), reqwest::StatusCode::OK);
        handle.abort();
    }
}
