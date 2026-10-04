use serde_json::{json, Value};

use super::array_mut;

pub(super) fn remap(
    value: &mut Value,
    selected: &[(String, String, String)],
) -> Result<(), String> {
    for row in array_mut(value, "voiceRevisions")? {
        let Some((_, source_id, target_id)) = selected.iter().find(|(revision_id, _, _)| {
            row.get("id").and_then(Value::as_str) == Some(revision_id.as_str())
        }) else {
            continue;
        };
        let authorization = row
            .get_mut("authorization")
            .and_then(Value::as_object_mut)
            .ok_or("Selected voice revision authorization is unavailable")?;
        if authorization.get("candidateId").and_then(Value::as_str) != Some(source_id)
            || authorization.get("candidateRunId").and_then(Value::as_str) != Some(source_id)
        {
            return Err("Selected voice revision candidate provenance is invalid".into());
        }
        authorization.insert("candidateId".into(), json!(target_id));
        authorization.insert("candidateRunId".into(), json!(target_id));
    }
    Ok(())
}
