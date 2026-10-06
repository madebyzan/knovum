use super::super::{
    connectors,
    contracts::{ConnectorAuthorization, ConnectorDefinition, McpConnection, McpEndpoint},
};
use futures::future::BoxFuture;
use rmcp::{
    model::{ClientInfo, ElicitRequestParams, ElicitResult, ProgressNotificationParam},
    service::{NotificationContext, RequestContext, RunningService},
    ClientHandler, RoleClient, ServiceExt,
};
use std::{sync::Arc, time::Duration};
use tauri::AppHandle;

pub(crate) trait McpInteraction: Send + Sync {
    fn elicit(
        &self,
        request: ElicitRequestParams,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> BoxFuture<'_, Result<ElicitResult, String>>;
    fn progress(&self, progress: ProgressNotificationParam) -> BoxFuture<'_, ()>;
}

#[derive(Clone)]
pub(super) struct ExtensionClient {
    interaction: Option<Arc<dyn McpInteraction>>,
}

impl ClientHandler for ExtensionClient {
    fn get_info(&self) -> ClientInfo {
        let mut info = ClientInfo::default();
        info.client_info = rmcp::model::Implementation::new("knovum", env!("CARGO_PKG_VERSION"));
        if self.interaction.is_some() {
            info.capabilities.elicitation = Some(
                rmcp::model::ElicitationCapability::default()
                    .with_form(rmcp::model::FormElicitationCapability::default())
                    .with_url(rmcp::model::UrlElicitationCapability::default()),
            );
        }
        info
    }
    async fn create_elicitation(
        &self,
        request: ElicitRequestParams,
        context: RequestContext<RoleClient>,
    ) -> Result<ElicitResult, rmcp::ErrorData> {
        if let ElicitRequestParams::FormElicitationParams {
            requested_schema, ..
        } = &request
        {
            let schema = serde_json::to_value(requested_schema)
                .map_err(|error| rmcp::ErrorData::invalid_params(error.to_string(), None))?;
            let fields = schema["properties"].as_object().ok_or_else(|| {
                rmcp::ErrorData::invalid_params("elicitation properties required", None)
            })?;
            if fields.keys().any(|name| super::endpoint::secret_name(name)) {
                return Ok(ElicitResult::new(rmcp::model::ElicitationAction::Decline));
            }
        }
        let interaction = self.interaction.as_ref().ok_or_else(|| {
            rmcp::ErrorData::invalid_request(
                "elicitation is unavailable during a connection check",
                None,
            )
        })?;
        interaction
            .elicit(request, context.ct)
            .await
            .map_err(|error| rmcp::ErrorData::internal_error(error, None))
    }
    async fn on_progress(
        &self,
        progress: ProgressNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) {
        if let Some(interaction) = &self.interaction {
            interaction.progress(progress).await;
        }
    }
}

pub(crate) struct McpSession {
    pub(super) service: RunningService<RoleClient, ExtensionClient>,
    process: Option<super::stdio::StdioSession>,
}

impl McpSession {
    pub(crate) fn cancel(&self) {
        self.service.cancellation_token().cancel();
        if let Some(process) = &self.process {
            process.tree.handle().terminate();
        }
    }

    pub(super) async fn close(mut self) {
        let _ = self
            .service
            .close_with_timeout(Duration::from_secs(2))
            .await;
        if let Some(process) = &mut self.process {
            process.tree.terminate();
            let _ = process.child.wait().await;
        }
    }
}

pub(super) async fn connect(
    app: &AppHandle,
    connection: &McpConnection,
    connector: &ConnectorDefinition,
    interaction: Option<Arc<dyn McpInteraction>>,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<McpSession, String> {
    let pool = super::super::database::pool(app).await?;
    connectors::definitions::require_current(&pool, connector).await?;
    let endpoint = connector.configured_endpoint()?;
    connectors::requirements::validate_access(endpoint, &connector.authorization)?;
    if let Some(id) = &connector.plugin_id {
        if !super::super::plugins::load(&pool, id).await?.enabled {
            return Err("mcp-plugin-disabled".into());
        }
    }
    let account = if let Some(id) = &connection.account_id {
        let account = connectors::accounts::load(&pool, id).await?;
        connectors::accounts::validate_binding(connector, &account)?;
        Some(account)
    } else {
        if !matches!(connector.authorization, ConnectorAuthorization::None) {
            return Err("connector-authorized-account-required".into());
        }
        None
    };
    let handler = ExtensionClient { interaction };
    let (service, process) = match endpoint {
        McpEndpoint::StreamableHttp {
            url,
            headers,
            allow_local_network,
        } => {
            use rmcp::transport::{
                streamable_http_client::StreamableHttpClientTransportConfig,
                StreamableHttpClientTransport,
            };
            let client = super::http_client::BoundedHttpClient(
                super::endpoint::http_client(url, *allow_local_network).await?,
            );
            let mut config = StreamableHttpClientTransportConfig::with_uri(url.as_str())
                .reinit_on_expired_session(false)
                .max_sse_event_size(8 * 1024 * 1024);
            let mut custom_headers = std::collections::HashMap::new();
            for (name, value) in headers {
                custom_headers.insert(
                    reqwest::header::HeaderName::from_bytes(name.as_bytes())
                        .map_err(|_| "mcp-header-name-invalid")?,
                    reqwest::header::HeaderValue::from_str(value)
                        .map_err(|_| "mcp-header-value-invalid")?,
                );
            }
            let service = if account.as_ref().is_some_and(|account| {
                matches!(account.authorization, ConnectorAuthorization::OAuth { .. })
            }) {
                let account = account.as_ref().expect("OAuth account");
                config.custom_headers = custom_headers;
                let client = connectors::authorization::oauth_client(
                    app,
                    account,
                    client,
                    *allow_local_network,
                )
                .await?;
                tokio::time::timeout(
                    Duration::from_secs(30),
                    handler.serve_with_ct(
                        StreamableHttpClientTransport::with_client(client, config),
                        cancellation,
                    ),
                )
                .await
                .map_err(|_| "mcp-handshake-timeout")?
                .map_err(|_| "mcp-handshake-failed")?
            } else {
                if let Some(id) = &connection.account_id {
                    for (name, value) in connectors::credentials(app, id, url).await? {
                        if super::endpoint::reserved_header(&name) {
                            return Err("connector-cannot-override-mcp-transport-header".into());
                        }
                        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                            .map_err(|_| "connector-secret-header-name-invalid")?;
                        let mut value = reqwest::header::HeaderValue::from_str(&value)
                            .map_err(|_| "connector-secret-header-value-invalid")?;
                        value.set_sensitive(true);
                        custom_headers.insert(name, value);
                    }
                }
                config.custom_headers = custom_headers;
                tokio::time::timeout(
                    Duration::from_secs(30),
                    handler.serve_with_ct(
                        StreamableHttpClientTransport::with_client(client, config),
                        cancellation,
                    ),
                )
                .await
                .map_err(|_| "mcp-handshake-timeout")?
                .map_err(|_| "mcp-handshake-failed")?
            };
            (service, None)
        }
        McpEndpoint::Stdio {
            executable,
            args,
            cwd,
            environment,
        } => {
            let plugin_environment =
                super::super::plugins::process_configuration::environment(app, connector).await?;
            let secrets = if let Some(id) = &connection.account_id {
                connectors::credentials(app, id, &super::endpoint::resource(endpoint)).await?
            } else {
                Default::default()
            };
            let mut environment = environment.clone();
            environment.extend(plugin_environment);
            let (process, stdout, stdin) =
                super::stdio::start(executable, args, cwd.as_deref(), &environment, &secrets)?;
            let service = tokio::time::timeout(
                Duration::from_secs(30),
                handler.serve_with_ct(super::stdio::transport(stdout, stdin), cancellation),
            )
            .await
            .map_err(|_| "mcp-handshake-timeout")?
            .map_err(|_| "mcp-handshake-failed")?;
            (service, Some(process))
        }
    };
    Ok(McpSession { service, process })
}

#[cfg(test)]
#[path = "tests/client.rs"]
mod tests;
