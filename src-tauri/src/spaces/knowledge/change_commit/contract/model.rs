use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", rename_all_fields = "camelCase")]
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
        locator: Value,
        source_revision: String,
        quote: String,
    },
}

#[derive(Debug, Deserialize)]
pub(crate) struct Position {
    pub(crate) x: f64,
    pub(crate) y: f64,
}
