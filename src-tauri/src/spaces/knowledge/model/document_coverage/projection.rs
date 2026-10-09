use serde_json::{json, Value};

pub(crate) fn semantic_content(payload: &Value) -> Value {
    let occurrences = payload
        .get("documentCoverage")
        .and_then(|value| value.get("occurrences"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|value| !value.get("analysis").is_none_or(Value::is_null))
        .map(|value| {
            json!({"id":value["id"], "location":value["location"], "analysis":value["analysis"]})
        })
        .collect::<Vec<_>>();
    json!(occurrences)
}
