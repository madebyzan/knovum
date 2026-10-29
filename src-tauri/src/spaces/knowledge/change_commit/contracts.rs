use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// One operation of a Knowledge Change Set. The derived JSON Schema is what Agents are given to
// author a Change Set, so it states every operation exactly as this type accepts it. It serializes
// to the same shape, which is how a pending Knowledge Proposal stores and presents it.
#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[ts(optional_fields = nullable)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum KnowledgeChange {
    CreateObject {
        temp_id: String,
        object_type_id: String,
        title: String,
        summary: Option<String>,
        status: String,
        properties: Map<String, Value>,
        position: Option<Position>,
    },
    UpdateObject {
        node_id: String,
        title: Option<String>,
        summary: Option<String>,
        status: Option<String>,
        properties: Option<Map<String, Value>>,
    },
    CreateRelation {
        temp_id: String,
        source_id: String,
        target_id: String,
        kind: String,
        rationale: String,
        confidence: Option<f64>,
    },
    AddProvenance {
        target_kind: String,
        target_id: String,
        source_node_id: String,
        #[ts(type = "unknown")]
        #[schemars(
            with = "Map<String, Value>",
            description = "Knowledge locator object with sourceRevision, location and optional quote"
        )]
        locator: Value,
        source_revision: String,
        quote: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS, schemars::JsonSchema)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct Position {
    pub(crate) x: f64,
    pub(crate) y: f64,
}
