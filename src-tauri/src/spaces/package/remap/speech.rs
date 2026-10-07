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
    for row in array_mut(value, "speechSourceSeparations")? {
        let id = row.get("id").and_then(Value::as_str);
        let run_id = row.get("runId").and_then(Value::as_str);
        if id.is_none() || id != run_id {
            return Err("Package source separation identity must equal its Run identity".into());
        }
        maps.field(row, "runId", "runs")?;
        row["id"] = row["runId"].clone();
        row["sourceAssetPath"] = json!(maps.asset(
            row.get("sourceAssetPath")
                .and_then(Value::as_str)
                .ok_or("Package source separation video asset is unavailable")?,
        )?);
        row["backgroundAssetPath"] = json!(maps.asset(
            row.get("backgroundAssetPath")
                .and_then(Value::as_str)
                .ok_or("Package source separation background asset is unavailable")?,
        )?);
    }
    for row in array_mut(value, "speechProjects")? {
        maps.field(row, "sourceNodeId", "nodes")?;
        maps.optional(row, "activeRevisionId", "speechProjectRevisions")?;
    }
    for row in array_mut(value, "speechProjectRevisions")? {
        maps.field(row, "projectId", "speechProjects")?;
        if let Some(config) = row.get_mut("outputConfig") {
            if let Some(roles) = config.get_mut("roles").and_then(Value::as_array_mut) {
                for role in roles {
                    maps.field(role, "voiceRevisionId", "voiceRevisions")?;
                }
            }
            if let Some(separation_id) = config
                .get("backgroundSeparation")
                .and_then(|source| source.get("separationId"))
                .and_then(Value::as_str)
            {
                config["backgroundSeparation"]["separationId"] =
                    json!(maps.id("runs", separation_id)?);
            }
            if let Some(reference) = config
                .get("backgroundSeparation")
                .and_then(|separation| separation.get("source"))
                .and_then(|source| source.get("assetPath"))
                .and_then(Value::as_str)
            {
                config["backgroundSeparation"]["source"]["assetPath"] =
                    json!(maps.asset(reference)?);
            }
            if let Some(reference) = config
                .get("metadata")
                .and_then(|metadata| metadata.get("cover"))
                .and_then(|cover| cover.get("assetPath"))
                .and_then(Value::as_str)
            {
                config["metadata"]["cover"]["assetPath"] = json!(maps.asset(reference)?);
            }
            for key in ["video", "background"] {
                if let Some(reference) = config
                    .get(key)
                    .and_then(|source| source.get("assetPath"))
                    .and_then(Value::as_str)
                {
                    config[key]["assetPath"] = json!(maps.asset(reference)?);
                }
            }
        }
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
