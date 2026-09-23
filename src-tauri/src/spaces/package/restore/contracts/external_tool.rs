use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageExternalToolBindingRow {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub server_identity: Value,
    pub tool_name: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub schema_fingerprint: String,
    pub effect: String,
    pub reviewed_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
