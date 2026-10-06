use super::super::{content, contracts::McpEndpoint};
use std::time::Duration;

pub(crate) fn resource(endpoint: &McpEndpoint) -> String {
    match endpoint {
        McpEndpoint::StreamableHttp { url, .. } => url.clone(),
        McpEndpoint::Stdio { .. } => format!(
            "stdio:{}",
            content::digest(&serde_json::to_vec(endpoint).expect("owned endpoint serializes"))
        ),
    }
}

pub(in crate::extensions) fn validate(endpoint: &McpEndpoint) -> Result<(), String> {
    match endpoint {
        McpEndpoint::StreamableHttp { url, headers, .. } => {
            let parsed = url::Url::parse(url).map_err(|error| error.to_string())?;
            if !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.fragment().is_some()
                || parsed.query_pairs().any(|(name, _)| secret_name(&name))
            {
                return Err("mcp-url-must-not-contain-credentials-or-fragment".into());
            }
            let mut names = std::collections::BTreeSet::new();
            for (name, value) in headers {
                if !names.insert(name.to_ascii_lowercase()) {
                    return Err("mcp-header-name-duplicated".into());
                }
                reqwest::header::HeaderValue::from_str(value)
                    .map_err(|_| "mcp-header-value-invalid")?;
                if reserved_header(name) {
                    return Err("mcp-transport-header-is-client-managed".into());
                }
                if secret_name(name) {
                    return Err("mcp-secret-headers-belong-to-connector-account".into());
                }
                reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| "mcp-header-name-invalid")?;
            }
        }
        McpEndpoint::Stdio {
            executable,
            args,
            cwd,
            environment,
        } => {
            if args.iter().any(|value| value.contains('\0'))
                || environment.iter().any(|(name, value)| {
                    name.is_empty() || name.contains(['=', '\0']) || value.contains('\0')
                })
            {
                return Err("mcp-process-field-invalid".into());
            }
            if cwd.as_ref().is_some_and(|cwd| {
                !std::path::Path::new(cwd).is_absolute() || !std::path::Path::new(cwd).is_dir()
            }) {
                return Err("mcp-working-directory-must-be-existing-absolute-directory".into());
            }
            let path = std::path::Path::new(executable);
            if !path.is_absolute() || !path.is_file() {
                return Err("mcp-executable-requires-explicit-installed-absolute-path".into());
            }
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if matches!(
                stem.as_str(),
                "npx"
                    | "uvx"
                    | "pip"
                    | "pip3"
                    | "bash"
                    | "sh"
                    | "zsh"
                    | "cmd"
                    | "powershell"
                    | "pwsh"
            ) {
                return Err("mcp-package-installers-and-shells-not-authorized".into());
            }
            if environment.keys().any(|name| secret_name(name)) {
                return Err("mcp-secret-environment-belongs-to-connector-account".into());
            }
        }
    }
    Ok(())
}

pub(in crate::extensions) fn secret_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase().replace('-', "_");
    matches!(
        name.as_str(),
        "authorization"
            | "cookie"
            | "proxy_authorization"
            | "token"
            | "secret"
            | "password"
            | "api_key"
            | "private_key"
            | "access_key"
            | "apikey"
            | "accesstoken"
            | "refreshtoken"
            | "clientsecret"
            | "privatekey"
            | "secretaccesskey"
    ) || [
        "_token",
        "_secret",
        "_password",
        "_api_key",
        "_private_key",
        "_access_key",
        "_secret_access_key",
    ]
    .iter()
    .any(|suffix| name.ends_with(suffix))
}

pub(crate) async fn http_client(value: &str, allow_local: bool) -> Result<reqwest::Client, String> {
    let url = url::Url::parse(value).map_err(|error| error.to_string())?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err("mcp-url-credentials-forbidden".into());
    }
    let host = url
        .host_str()
        .ok_or("mcp-host-required")?
        .trim_matches(['[', ']']);
    let addresses = tokio::net::lookup_host((
        host,
        url.port_or_known_default().ok_or("mcp-port-required")?,
    ))
    .await
    .map_err(|error| error.to_string())?
    .collect::<Vec<_>>();
    let private = addresses
        .iter()
        .any(|address| !crate::public_network::public_address(address.ip()));
    if addresses.is_empty()
        || (private && !allow_local)
        || (url.scheme() != "https" && !(url.scheme() == "http" && allow_local && private))
    {
        return Err("mcp-endpoint-requires-https-or-explicit-local-network-access".into());
    }
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve_to_addrs(host, &addresses)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| error.to_string())
}

pub(in crate::extensions) fn reserved_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "accept"
            | "content-type"
            | "mcp-session-id"
            | "mcp-protocol-version"
            | "last-event-id"
            | "host"
            | "content-length"
            | "transfer-encoding"
    )
}

#[cfg(test)]
#[path = "tests/endpoint.rs"]
mod tests;
