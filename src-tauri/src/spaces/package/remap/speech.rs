use super::{array_mut, IdMaps};
use serde_json::{json, Value};

pub(super) fn remap(value: &mut Value, maps: &IdMaps) -> Result<(), String> {
    for row in array_mut(value, "voiceProfiles")? {
        maps.optional(row, "activeRevisionId", "voiceRevisions")?;
    }
    for row in array_mut(value, "voiceRevisions")? {
        maps.field(row, "profileId", "voiceProfiles")?;
        if let Some(reference) = row.get("referenceAssetId").and_then(Value::as_str) {
            row["referenceAssetId"] = json!(maps.asset(reference)?);
        }
    }
    for row in array_mut(value, "speechProjects")? {
        maps.optional(row, "activeRevisionId", "speechProjectRevisions")?;
    }
    for row in array_mut(value, "speechProjectRevisions")? {
        maps.field(row, "projectId", "speechProjects")?;
        if let Some(reference) = row.get("outputAssetPath").and_then(Value::as_str) {
            row["outputAssetPath"] = json!(maps.asset(reference)?);
        }
    }
    for row in array_mut(value, "speechSegments")? {
        maps.field(row, "projectRevisionId", "speechProjectRevisions")?;
        maps.field(row, "voiceRevisionId", "voiceRevisions")?;
    }
    for row in array_mut(value, "speechRenders")? {
        maps.field(row, "segmentId", "speechSegments")?;
        if let Some(reference) = row.get("assetPath").and_then(Value::as_str) {
            row["assetPath"] = json!(maps.asset(reference)?);
        }
    }
    Ok(())
}
