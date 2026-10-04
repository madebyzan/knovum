use serde_json::{json, Value};
use std::collections::HashSet;

use super::{array_mut, IdMaps};

const WORKFLOW: &str = "speech_video_translation_v1";
const NODE_KIND: &str = "speech_video_translation_segment";

pub(super) fn remap(value: &mut Value, maps: &IdMaps) -> Result<(), String> {
    for run in array_mut(value, "runs")? {
        let Some(input) = run
            .get_mut("inputSnapshot")
            .filter(|input| input.get("workflow").and_then(Value::as_str) == Some(WORKFLOW))
        else {
            continue;
        };
        maps.field(input, "projectId", "speechProjects")?;
        maps.field(input, "revisionId", "speechProjectRevisions")?;
    }
    let mut step_ids = HashSet::new();
    for step in array_mut(value, "runSteps")? {
        if step.get("nodeKind").and_then(Value::as_str) != Some(NODE_KIND) {
            continue;
        }
        {
            let configuration = step
                .get_mut("configuration")
                .ok_or("Package video translation step configuration is unavailable")?;
            maps.field(configuration, "projectId", "speechProjects")?;
            maps.field(configuration, "revisionId", "speechProjectRevisions")?;
        }
        let step_id = step["id"].clone();
        let step_id_text = step_id
            .as_str()
            .ok_or("Package video translation step identity is unavailable")?
            .to_owned();
        step_ids.insert(step_id_text);
        if let Some(output) = step.get_mut("output").filter(|output| !output.is_null()) {
            output["stepId"] = step_id;
        }
    }
    for attempt in array_mut(value, "runAttempts")? {
        let Some(step_id) = attempt.get("stepInstanceId").and_then(Value::as_str) else {
            continue;
        };
        if !step_ids.contains(step_id) {
            continue;
        }
        let step_id = step_id.to_owned();
        if let Some(result) = attempt.get_mut("result").filter(|result| !result.is_null()) {
            result["stepId"] = json!(step_id);
        }
    }
    for revision in array_mut(value, "speechProjectRevisions")? {
        let Some(translation) = revision
            .get_mut("outputConfig")
            .and_then(|config| config.get_mut("translation"))
            .filter(|translation| !translation.is_null())
        else {
            continue;
        };
        let source = translation
            .get("runId")
            .and_then(Value::as_str)
            .ok_or("Package video translation provenance Run is unavailable")?;
        translation["runId"] = json!(maps.id("runs", source)?);
    }
    Ok(())
}
