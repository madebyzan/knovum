use std::time::Duration;

use rmcp::{
    model::{CallToolRequestParams, ClientInfo},
    transport::{
        streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport,
    },
    ServiceExt,
};
use serde_json::{Map, Value};

pub(super) struct DiscoveredTool {
    pub server_identity: Value,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
}

pub(super) async fn discover(
    endpoint: &str,
    secret_key: Option<&str>,
    tool_name: &str,
) -> Result<DiscoveredTool, String> {
    let endpoint = validate_endpoint(endpoint).await?;
    let transport = StreamableHttpClientTransport::with_client(
        endpoint.client,
        config(&endpoint.url, secret_key).await?,
    );
    let service = tokio::time::timeout(
        Duration::from_secs(15),
        ClientInfo::default().serve(transport),
    )
    .await
    .map_err(|_| "external-tool-timeout: MCP initialization timed out")?
    .map_err(protocol_error)?;
    let identity = service
        .peer_info()
        .and_then(|info| info.server_info.clone())
        .ok_or("external-tool-identity-missing: MCP server did not declare an identity")?;
    let tools = tokio::time::timeout(Duration::from_secs(15), service.list_all_tools())
        .await
        .map_err(|_| "external-tool-timeout: MCP discovery timed out")?
        .map_err(protocol_error)?;
    let tool = tools
        .into_iter()
        .find(|tool| tool.name == tool_name)
        .ok_or_else(|| format!("external-tool-unavailable: tool {tool_name} was not advertised"))?;
    let discovered = DiscoveredTool {
        server_identity: serde_json::to_value(identity).map_err(crate::spaces::db::json_error)?,
        input_schema: Value::Object((*tool.input_schema).clone()),
        output_schema: tool
            .output_schema
            .map(|schema| Value::Object((*schema).clone())),
    };
    service.cancel().await.map_err(|error| error.to_string())?;
    Ok(discovered)
}

pub(super) async fn call(
    endpoint: &str,
    secret_key: Option<&str>,
    tool_name: &str,
    arguments: Map<String, Value>,
) -> Result<Value, String> {
    let endpoint = validate_endpoint(endpoint).await?;
    let transport = StreamableHttpClientTransport::with_client(
        endpoint.client,
        config(&endpoint.url, secret_key).await?,
    );
    let service = tokio::time::timeout(
        Duration::from_secs(15),
        ClientInfo::default().serve(transport),
    )
    .await
    .map_err(|_| "external-tool-timeout: MCP initialization timed out")?
    .map_err(protocol_error)?;
    let result = tokio::time::timeout(
        Duration::from_secs(120),
        service
            .call_tool(CallToolRequestParams::new(tool_name.to_owned()).with_arguments(arguments)),
    )
    .await
    .map_err(|_| "external-tool-timeout: MCP tool call timed out")?
    .map_err(protocol_error)?;
    service.cancel().await.map_err(|error| error.to_string())?;
    if result.is_error == Some(true) {
        return Err("external-tool-failed: MCP server returned a tool error".into());
    }
    serde_json::to_value(result).map_err(crate::spaces::db::json_error)
}

async fn config(
    endpoint: &str,
    secret_key: Option<&str>,
) -> Result<StreamableHttpClientTransportConfig, String> {
    let mut config = StreamableHttpClientTransportConfig::with_uri(endpoint.to_owned())
        .max_sse_event_size(MAX_RESULT_BYTES);
    if let Some(key) = secret_key.filter(|key| !key.trim().is_empty()) {
        let token = crate::secrets::read_secret(key)
            .await?
            .ok_or("external-tool-credential-missing: configured credential is unavailable")?;
        config = config.auth_header(token);
    }
    Ok(config)
}

const MAX_RESULT_BYTES: usize = 1_048_576;

struct ResolvedEndpoint {
    url: String,
    client: reqwest::Client,
}

async fn validate_endpoint(value: &str) -> Result<ResolvedEndpoint, String> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| "external-tool-endpoint-invalid: endpoint is not a URL")?;
    if url.username() != "" || url.password().is_some() || url.fragment().is_some() {
        return Err(
            "external-tool-endpoint-invalid: credentials and fragments are forbidden".into(),
        );
    }
    let host = url.host_str().unwrap_or_default();
    let local_http = url.scheme() == "http" && matches!(host, "localhost" | "127.0.0.1" | "::1");
    if url.scheme() != "https" && !local_http {
        return Err(
            "external-tool-endpoint-invalid: use HTTPS or an explicit loopback endpoint".into(),
        );
    }
    let mut client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none());
    if !local_http {
        let port = url
            .port_or_known_default()
            .ok_or("external-tool-endpoint-invalid: endpoint port is unavailable")?;
        let addresses = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| "external-tool-endpoint-unresolved: endpoint host cannot be resolved")?;
        let addresses = addresses.collect::<Vec<_>>();
        if addresses.is_empty()
            || addresses
                .iter()
                .any(|address| is_private_address(address.ip()))
        {
            return Err("external-tool-endpoint-forbidden: remote tools cannot target local or private networks".into());
        }
        client = client.resolve_to_addrs(host, &addresses);
    }
    Ok(ResolvedEndpoint {
        url: url.to_string(),
        client: client
            .build()
            .map_err(|error| format!("external-tool-client-invalid: {error}"))?,
    })
}

fn is_private_address(address: std::net::IpAddr) -> bool {
    match address {
        std::net::IpAddr::V4(address) => {
            address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_broadcast()
                || address.is_unspecified()
        }
        std::net::IpAddr::V6(address) => {
            address.is_loopback()
                || address.is_unspecified()
                || address.is_unique_local()
                || address.is_unicast_link_local()
        }
    }
}

fn protocol_error(error: impl std::fmt::Display) -> String {
    format!("external-tool-protocol-error: {error}")
}
