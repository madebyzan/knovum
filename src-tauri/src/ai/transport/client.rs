use std::sync::OnceLock;
use std::time::Duration;

pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(60))
            .tcp_keepalive(Duration::from_secs(30))
            .timeout(Duration::from_secs(180))
            .build()
            .expect("the static AI HTTP client configuration must be valid")
    })
}
