//! Product identification for Knovum-owned outbound HTTP clients; callers own transfer policy.

// Unidentified traffic is rejected by CDN edge UA ACLs, so every client identifies the product.
pub(crate) const USER_AGENT: &str = concat!("Knovum/", env!("CARGO_PKG_VERSION"));

pub(crate) fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder().user_agent(USER_AGENT)
}

#[cfg(test)]
mod tests;
