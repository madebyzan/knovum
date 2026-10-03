use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RelatedContentItem {
    pub node_id: String,
    pub node_kind: String,
    pub node_title: String,
    pub snippet: String,
    pub score: f64,
}

pub(super) fn collect_match(
    by_node: &mut HashMap<String, RelatedContentItem>,
    value: &Value,
    source_node_id: &str,
) -> Result<(), String> {
    let field = |name: &str| {
        value
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Related content result is missing {name}"))
    };
    let node_id = field("nodeId")?;
    if node_id == source_node_id {
        return Ok(());
    }
    let score = value
        .get("score")
        .and_then(Value::as_f64)
        .ok_or_else(|| "Related content result is missing score".to_owned())?;
    let current = by_node.get(node_id);
    if current.is_some_and(|item| item.score >= score) {
        return Ok(());
    }
    by_node.insert(
        node_id.to_owned(),
        RelatedContentItem {
            node_id: node_id.to_owned(),
            node_kind: field("nodeKind")?.to_owned(),
            node_title: field("nodeTitle")?.to_owned(),
            snippet: field("snippet")?.to_owned(),
            score,
        },
    );
    Ok(())
}
