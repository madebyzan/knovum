use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};

use super::{array_mut, voice_candidate_selection, IdMaps};

const NODE_KIND: &str = "speech_voice_candidate";

pub(super) fn remap(value: &mut Value, maps: &IdMaps) -> Result<(), String> {
    let selected = remap_candidates(value, maps)?;
    voice_candidate_selection::remap(value, &selected)?;
    let runs = run_sources(maps)?;
    let steps = remap_steps(value, maps, &runs)?;
    remap_attempts(value, maps, &steps, &runs)
}

fn remap_candidates(
    value: &mut Value,
    maps: &IdMaps,
) -> Result<Vec<(String, String, String)>, String> {
    let mut selected = Vec::new();
    for row in array_mut(value, "voiceCandidates")? {
        if row.get("id").and_then(Value::as_str) != row.get("runId").and_then(Value::as_str) {
            return Err("Package voice candidate identity must equal its Run identity".into());
        }
        let source_id = required(row, "id")?.to_owned();
        maps.field(row, "runId", "runs")?;
        row["id"] = row["runId"].clone();
        let target_id = required(row, "id")?.to_owned();
        for field in ["referenceAssetPath", "previewAssetPath"] {
            let reference = row
                .get(field)
                .and_then(Value::as_str)
                .ok_or("Package voice candidate asset is unavailable")?;
            row[field] = json!(maps.asset(reference)?);
        }
        maps.optional(row, "selectedVoiceProfileId", "voiceProfiles")?;
        maps.optional(row, "selectedVoiceRevisionId", "voiceRevisions")?;
        if row.get("status").and_then(Value::as_str) == Some("selected") {
            selected.push((
                required(row, "selectedVoiceRevisionId")?.into(),
                source_id,
                target_id,
            ));
        }
    }
    Ok(selected)
}

fn remap_steps(
    value: &mut Value,
    maps: &IdMaps,
    runs: &HashMap<String, String>,
) -> Result<HashSet<String>, String> {
    let mut steps = HashSet::new();
    for step in array_mut(value, "runSteps")? {
        if step.get("nodeKind").and_then(Value::as_str) != Some(NODE_KIND) {
            continue;
        }
        let run_id = required(step, "runId")?.to_owned();
        let source_id = source_run_id(runs, &run_id)?;
        steps.insert(required(step, "id")?.to_owned());
        let request = step
            .get_mut("configuration")
            .and_then(|configuration| configuration.get_mut("request"))
            .ok_or("Package voice candidate request snapshot is unavailable")?;
        require_historical_identity(request, source_id, &maps.source_space)?;
        request["requestId"] = json!(run_id);
        request["spaceId"] = json!(maps.target_space);
        if let Some(output) = step.get_mut("output").filter(|output| !output.is_null()) {
            require_historical_identity(output, source_id, &maps.source_space)?;
            remap_output(output, &run_id, &maps.target_space)?;
        }
    }
    Ok(steps)
}

fn remap_attempts(
    value: &mut Value,
    maps: &IdMaps,
    steps: &HashSet<String>,
    runs: &HashMap<String, String>,
) -> Result<(), String> {
    for attempt in array_mut(value, "runAttempts")? {
        let step_id = required(attempt, "stepInstanceId")?.to_owned();
        if !steps.contains(&step_id) {
            continue;
        }
        attempt["idempotencyKey"] = json!(step_id);
        let run_id = required(attempt, "runId")?.to_owned();
        let source_id = source_run_id(runs, &run_id)?;
        if let Some(output) = attempt.get_mut("output").filter(|output| !output.is_null()) {
            require_historical_identity(output, source_id, &maps.source_space)?;
            remap_output(output, &run_id, &maps.target_space)?;
        }
    }
    Ok(())
}

fn require_historical_identity(value: &Value, candidate: &str, space: &str) -> Result<(), String> {
    let identity_matches = match (
        field(value, "requestId"),
        field(value, "candidateId"),
        field(value, "runId"),
    ) {
        (Some(request), None, None) => request == candidate,
        (None, Some(output), Some(run)) => output == candidate && run == candidate,
        _ => false,
    };
    if !identity_matches || field(value, "spaceId") != Some(space) {
        return Err("Package voice candidate historical identity is invalid".into());
    }
    Ok(())
}

fn run_sources(maps: &IdMaps) -> Result<HashMap<String, String>, String> {
    Ok(maps
        .entities
        .get("runs")
        .ok_or("Package Run remaps are unavailable")?
        .iter()
        .map(|(source, target)| (target.clone(), source.clone()))
        .collect())
}

fn source_run_id<'a>(runs: &'a HashMap<String, String>, target: &str) -> Result<&'a str, String> {
    runs.get(target)
        .map(String::as_str)
        .ok_or_else(|| "Package voice candidate Run remap is unavailable".into())
}

fn remap_output(output: &mut Value, run_id: &str, space_id: &str) -> Result<(), String> {
    let object = output
        .as_object_mut()
        .ok_or("Package voice candidate output must be an object")?;
    object.insert("candidateId".into(), json!(run_id));
    object.insert("runId".into(), json!(run_id));
    object.insert("spaceId".into(), json!(space_id));
    Ok(())
}

fn required<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Package voice candidate {field} is unavailable"))
}

fn field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}
