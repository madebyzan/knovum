//! Every MCP endpoint handed to an Agent is a Knovum-owned server the Agent must reach directly.
//!
//! Agent processes inherit the desktop environment. An inherited `HTTP(S)_PROXY` routes the
//! Agent's requests for those endpoints to the proxy instead, and the Agent still opens its
//! session — only without the tools. The launch environment therefore extends the proxy
//! exemption list with the host of every endpoint, keeping the user's own entries.

use agent_client_protocol::schema::v1::McpServer;

use crate::foundation::app_error::{AppError, AppResult};

/// Agents read either spelling (undici and Go read both, curl only the lowercase one), so both
/// carry the same list.
const PROXY_EXEMPTIONS: [&str; 2] = ["NO_PROXY", "no_proxy"];

pub(super) fn environment(
    servers: &[McpServer],
    inherited: impl Fn(&str) -> Option<String>,
) -> AppResult<Vec<(&'static str, String)>> {
    let hosts = servers
        .iter()
        .filter_map(|server| match server {
            McpServer::Http(server) => Some(server.url.as_str()),
            McpServer::Sse(server) => Some(server.url.as_str()),
            _ => None,
        })
        .map(endpoint_host)
        .collect::<AppResult<Vec<_>>>()?;
    if hosts.is_empty() {
        return Ok(Vec::new());
    }
    let user_lists = PROXY_EXEMPTIONS
        .iter()
        .filter_map(|name| inherited(name))
        .collect::<Vec<_>>();
    let mut exemptions = Vec::<String>::new();
    for entry in user_lists
        .iter()
        .flat_map(|value| value.split(','))
        .map(|entry| entry.trim().to_owned())
        .chain(hosts)
    {
        if !entry.is_empty() && !exemptions.contains(&entry) {
            exemptions.push(entry);
        }
    }
    let exemptions = exemptions.join(",");
    Ok(PROXY_EXEMPTIONS
        .iter()
        .map(|name| (*name, exemptions.clone()))
        .collect())
}

fn endpoint_host(endpoint: &str) -> AppResult<String> {
    url::Url::parse(endpoint)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .ok_or_else(|| AppError::invalid(format!("MCP endpoint has no host: {endpoint}")))
}

#[cfg(test)]
mod tests;
