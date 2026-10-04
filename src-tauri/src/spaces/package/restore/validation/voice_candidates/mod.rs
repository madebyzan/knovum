use std::collections::HashMap;

use serde_json::Value;

use super::super::contracts::{
    PackageRun, PackageRunAttempt, PackageRunStep, RestorePackageRequest,
};

mod evidence;
mod provenance;
#[cfg(test)]
mod tests;

const WORKFLOW: &str = "speech_voice_candidate_v1";
const NODE_KIND: &str = "speech_voice_candidate";

pub(super) fn validate(request: &RestorePackageRequest) -> Result<(), String> {
    let runs = request
        .runs
        .iter()
        .filter(|run| workflow(run) == Some(WORKFLOW))
        .map(|run| (run.id.as_str(), run))
        .collect::<HashMap<_, _>>();
    let mut steps = HashMap::new();
    for step in request
        .run_steps
        .iter()
        .filter(|step| step.node_kind == NODE_KIND)
    {
        if !runs.contains_key(step.run_id.as_str())
            || steps.insert(step.run_id.as_str(), step).is_some()
        {
            return Err("Package voice candidate step ownership is invalid".into());
        }
    }
    for run in runs.values() {
        let candidate = request
            .voice_candidates
            .iter()
            .find(|candidate| candidate.run_id == run.id);
        if run.state == "succeeded" && candidate.is_none() {
            return Err("Succeeded package voice candidate Run has no candidate".into());
        }
        if run.state != "succeeded" && candidate.is_some() {
            return Err("Package voice candidate belongs to a non-succeeded Run".into());
        }
    }
    for candidate in &request.voice_candidates {
        let run = runs
            .get(candidate.run_id.as_str())
            .ok_or("Package voice candidate references another workflow")?;
        let step = steps
            .get(candidate.run_id.as_str())
            .ok_or("Package voice candidate step is unavailable")?;
        if request
            .run_steps
            .iter()
            .filter(|candidate_step| candidate_step.run_id == run.id)
            .count()
            != 1
        {
            return Err("Package voice candidate Run must contain exactly one step".into());
        }
        let attempts = candidate_attempts(request, run, step)?;
        validate_run(run, step, attempts.len())?;
        provenance::validate(request, candidate, run, step, &attempts)?;
    }
    Ok(())
}

fn candidate_attempts<'a>(
    request: &'a RestorePackageRequest,
    run: &PackageRun,
    step: &PackageRunStep,
) -> Result<Vec<&'a PackageRunAttempt>, String> {
    let attempts = request
        .run_attempts
        .iter()
        .filter(|attempt| attempt.run_id == run.id || attempt.step_instance_id == step.id)
        .collect::<Vec<_>>();
    if attempts
        .iter()
        .any(|attempt| attempt.run_id != run.id || attempt.step_instance_id != step.id)
    {
        return Err("Package voice candidate attempt ownership is invalid".into());
    }
    let succeeded = attempts
        .iter()
        .filter(|attempt| attempt.state == "succeeded")
        .count();
    if succeeded != 1
        || attempts
            .iter()
            .any(|attempt| !matches!(attempt.state.as_str(), "succeeded" | "failed" | "cancelled"))
    {
        return Err("Package voice candidate attempts are incomplete".into());
    }
    Ok(attempts)
}

fn validate_run(run: &PackageRun, step: &PackageRunStep, attempts: usize) -> Result<(), String> {
    if run.input_snapshot.as_object().map(serde_json::Map::len) != Some(3)
        || run.kind != "workflow"
        || run.state != "succeeded"
        || run.expanded_steps != 1
        || run.step_attempts != i64::try_from(attempts).unwrap_or(-1)
        || step.definition_node_id != "candidate"
        || step.instance_path != "candidate"
        || step.state != "succeeded"
        || step.depends_on != serde_json::json!([])
        || step.resolved_input != serde_json::json!({})
        || step.output.is_none()
    {
        return Err("Package voice candidate Run structure is invalid".into());
    }
    let digest = field(&run.input_snapshot, "requestDigest")?;
    if !lower_hex64(digest) {
        return Err("Package voice candidate request digest is invalid".into());
    }
    Ok(())
}

pub(super) fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Package voice candidate {name} is unavailable"))
}

pub(super) fn lower_hex64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn workflow(run: &PackageRun) -> Option<&str> {
    run.input_snapshot.get("workflow").and_then(Value::as_str)
}
