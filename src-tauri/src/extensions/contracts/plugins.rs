use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PluginSource {
    Directory {
        path: String,
    },
    Zip {
        path: String,
    },
    HttpsZip {
        url: String,
        sha256: Option<String>,
    },
    Git {
        url: String,
        reference: String,
        subdirectory: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginManifest {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    #[ts(type = "unknown")]
    pub author: Option<Value>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub keywords: Option<Vec<String>>,
    #[ts(type = "Record<string, unknown> | null")]
    pub extensions: Option<std::collections::BTreeMap<String, Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginComponent {
    pub name: String,
    pub kind: String,
    pub supported: bool,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginConnectionDeclaration {
    pub name: String,
    #[ts(type = "unknown")]
    pub configuration: Value,
    pub authorization: super::ConnectorAuthorization,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginInspection {
    pub manifest: PluginManifest,
    pub digest: String,
    pub source_revision: String,
    pub components: Vec<PluginComponent>,
    pub connections: Vec<PluginConnectionDeclaration>,
    pub file_count: u32,
    pub byte_length: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginPublication {
    pub marketplace_id: String,
    pub publisher_id: String,
    pub plugin_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginInstallation {
    pub id: String,
    pub source: PluginSource,
    pub revision: PluginInspection,
    pub enabled: bool,
    pub builtin: bool,
    pub publication: Option<PluginPublication>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginRelease {
    pub publisher_id: String,
    pub name: String,
    pub description: String,
    pub source: PluginSource,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MarketplaceSource {
    Directory {
        path: String,
    },
    Https {
        url: String,
    },
    Git {
        url: String,
        reference: String,
        subdirectory: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginMarketplace {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub source: MarketplaceSource,
    pub releases: Vec<PluginRelease>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginUpdatePreview {
    pub source: PluginSource,
    pub inspection: PluginInspection,
}
