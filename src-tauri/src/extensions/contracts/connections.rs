use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum McpEndpoint {
    StreamableHttp {
        url: String,
        headers: BTreeMap<String, String>,
        allow_local_network: bool,
    },
    Stdio {
        executable: String,
        args: Vec<String>,
        cwd: Option<String>,
        environment: BTreeMap<String, String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ConnectorAuthorization {
    None,
    Bearer,
    SecretFields {
        names: Vec<String>,
    },
    #[serde(rename = "oauth")]
    OAuth {
        client_id: Option<String>,
        scopes: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectorDefinition {
    pub id: String,
    pub revision: String,
    pub name: String,
    pub description: String,
    pub plugin_id: Option<String>,
    pub component_name: Option<String>,
    pub endpoint: McpEndpoint,
    pub authorization: ConnectorAuthorization,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectorAccount {
    pub id: String,
    pub revision: String,
    pub connector_id: String,
    pub display_name: String,
    pub resource: String,
    pub issuer: Option<String>,
    pub scopes: Vec<String>,
    pub authorization: ConnectorAuthorization,
    pub enabled: bool,
    pub cleanup_pending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum McpToolPolicy {
    Allow,
    Confirm,
    Deny,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpToolPermission {
    pub name: String,
    pub schema_digest: String,
    pub read_only: bool,
    pub policy: McpToolPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpConnection {
    pub id: String,
    pub connector_id: String,
    pub name: String,
    pub revision: String,
    pub account_id: Option<String>,
    pub enabled: bool,
    pub tools: Vec<McpToolPermission>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpCatalogTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub schema_digest: String,
    pub read_only_hint: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpConnectionCatalog {
    pub server: Value,
    pub tools: Vec<McpCatalogTool>,
    // rmcp owns protocol parsing and serialization; these TS views expose fields used by the UI.
    #[ts(
        type = "Array<{ uri: string; name: string; title?: string; description?: string; mimeType?: string; size?: number }>"
    )]
    pub resources: Vec<rmcp::model::Resource>,
    #[ts(
        type = "Array<{ uriTemplate: string; name: string; title?: string; description?: string; mimeType?: string }>"
    )]
    pub resource_templates: Vec<rmcp::model::ResourceTemplate>,
    #[ts(
        type = "Array<{ name: string; title?: string; description?: string; arguments?: Array<{ name: string; title?: string; description?: string; required?: boolean }> }>"
    )]
    pub prompts: Vec<rmcp::model::Prompt>,
    pub digest: String,
}
