use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageAutomationTriggerRow {
    pub id: String,
    pub name: String,
    pub workflow_id: String,
    pub workflow_version: i64,
    pub source: Value,
    pub input: Value,
    pub overlap_policy: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageAutomationOccurrenceRow {
    pub id: String,
    pub trigger_id: String,
    pub scheduled_at: i64,
    pub occurrence_key: String,
    pub run_id: Option<String>,
    pub state: String,
    pub detail: Value,
    pub created_at: i64,
}
