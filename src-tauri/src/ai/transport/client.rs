use std::sync::OnceLock;
use std::time::Duration;

fn base_builder() -> reqwest::ClientBuilder {
    crate::foundation::http::client_builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(60))
        .tcp_keepalive(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("AI endpoint exceeded the redirect limit")
            } else if attempt.previous().first().is_some_and(|origin| {
                crate::foundation::public_network::same_origin(origin, attempt.url())
            }) {
                attempt.follow()
            } else {
                attempt.error("AI credentials cannot follow a redirect to another origin")
            }
        }))
}

/// Request/response HTTP client with a 3-minute total timeout for non-streaming AI calls.
pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        base_builder()
            .timeout(Duration::from_secs(180))
            .build()
            .expect("the static AI HTTP client configuration must be valid")
    })
}

/// Streaming HTTP client bounded by per-read idle timeout (3 minutes without new bytes) rather
/// than total elapsed time, so active long-form or deep-reasoning streams are never cut off mid-turn.
pub fn streaming_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        base_builder()
            .read_timeout(Duration::from_secs(180))
            .build()
            .expect("the static AI streaming HTTP client configuration must be valid")
    })
}
