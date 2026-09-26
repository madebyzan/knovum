use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Deserialize, ts_rs::TS)]
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
        locator: Value,
        source_revision: String,
        quote: String,
    },
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct Position {
    pub(crate) x: f64,
    pub(crate) y: f64,
}
