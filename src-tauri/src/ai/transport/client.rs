use std::sync::OnceLock;
use std::time::Duration;

pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        crate::foundation::http::client_builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(60))
            .tcp_keepalive(Duration::from_secs(30))
            .timeout(Duration::from_secs(180))
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
            .build()
            .expect("the static AI HTTP client configuration must be valid")
    })
}
