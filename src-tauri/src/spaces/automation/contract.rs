use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

const ACTION_EVENT_PREFIX: &str = "dev.knovum.space.action.";
const ACTION_EVENT_SUFFIX: &str = ".committed.v1";

#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct AutomationEventType(String);

impl AutomationEventType {
    pub fn action_committed(action_id: &str) -> Result<Self, String> {
        crate::spaces::actions::require(action_id)?;
        Ok(Self(format!(
            "{ACTION_EVENT_PREFIX}{action_id}{ACTION_EVENT_SUFFIX}"
        )))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn parse(value: String) -> Result<Self, String> {
        let action_id = value
            .strip_prefix(ACTION_EVENT_PREFIX)
            .and_then(|value| value.strip_suffix(ACTION_EVENT_SUFFIX))
            .ok_or("automation-event-type-invalid: expected a Knovum action event type")?;
        crate::spaces::actions::require(action_id)?;
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for AutomationEventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum OverlapPolicy {
    Skip,
    Queue,
    Parallel,
    CancelPrevious,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MisfirePolicy {
    Skip,
    Latest,
    CatchUp,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AutomationSource {
    Cron {
        schedule: String,
        timezone: String,
        misfire_policy: MisfirePolicy,
        misfire_window_seconds: u32,
        catch_up_limit: u8,
    },
    Event {
        #[ts(type = "string")]
        event_kind: AutomationEventType,
        filter_expression: Option<String>,
        causal_depth_limit: u8,
        rate_limit_count: u16,
        rate_limit_window_seconds: u32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export_to = "AutomationTrigger.ts")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AutomationTrigger {
    pub id: String,
    pub space_id: String,
    pub name: String,
    pub workflow_id: String,
    pub workflow_version: i64,
    pub grant_id: String,
    pub source: AutomationSource,
    pub input: Value,
    pub overlap_policy: OverlapPolicy,
    pub enabled: bool,
    pub valid_until: i64,
    pub next_scheduled_at: Option<i64>,
    pub last_scheduled_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export_to = "SaveAutomationTriggerRequest.ts")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveAutomationTriggerRequest {
    pub id: String,
    pub space_id: String,
    pub name: String,
    pub workflow_id: String,
    pub workflow_version: i64,
    pub source: AutomationSource,
    pub input: Value,
    pub overlap_policy: OverlapPolicy,
    pub enabled: bool,
    pub valid_until: i64,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "AutomationOccurrence.ts")]
#[serde(rename_all = "camelCase")]
pub struct AutomationOccurrence {
    pub id: String,
    pub trigger_id: String,
    pub scheduled_at: i64,
    pub occurrence_key: String,
    pub run_id: Option<String>,
    pub state: String,
    pub detail: Value,
    pub created_at: i64,
}
