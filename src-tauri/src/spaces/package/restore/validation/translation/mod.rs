use std::collections::HashMap;

use serde_json::Value;

use super::super::contracts::{
    PackageRun, PackageRunAttempt, PackageRunStep, RestorePackageRequest,
};

const WORKFLOW: &str = "speech_video_translation_v1";
const NODE_KIND: &str = "speech_video_translation_segment";

#[cfg(test)]
mod tests;

pub(super) fn validate(request: &RestorePackageRequest) -> Result<(), String> {
    let runs: HashMap<&str, &PackageRun> = request
        .runs
        .iter()
        .filter(|run| run.input_snapshot.get("workflow").and_then(Value::as_str) == Some(WORKFLOW))
        .map(|run| (run.id.as_str(), run))
        .collect();
    let mut steps: HashMap<&str, Vec<&PackageRunStep>> = HashMap::new();
    for step in request
        .run_steps
        .iter()
        .filter(|step| step.node_kind == NODE_KIND)
    {
        if !runs.contains_key(step.run_id.as_str()) {
            return Err("Package video translation step belongs to another workflow".into());
        }
        steps.entry(&step.run_id).or_default().push(step);
    }
    for run in runs.values().filter(|run| run.state == "succeeded") {
        let project_id = field(&run.input_snapshot, "projectId")?;
        let revision_id = field(&run.input_snapshot, "revisionId")?;
        let expected_revision = integer(&run.input_snapshot, "expectedRevision")?;
        let revision = request
            .speech_project_revisions
            .iter()
            .find(|revision| revision.id == revision_id && revision.project_id == project_id)
            .ok_or("Package video translation Run references an unknown project revision")?;
        if revision.revision != expected_revision {
            return Err("Package video translation Run base revision number differs".into());
        }
        validate_run(run, &revision.project_id)?;
        let run_steps = steps.get(run.id.as_str()).map(Vec::as_slice).unwrap_or(&[]);
        validate_steps(run, run_steps)?;
        validate_attempts(request, run, run_steps)?;
    }
    for revision in &request.speech_project_revisions {
        let Some(provenance) = revision
            .output_config
            .get("translation")
            .filter(|value| !value.is_null())
        else {
            continue;
        };
        let run_id = field(provenance, "runId")?;
        let run = runs
            .get(run_id)
            .ok_or("Package video translation provenance references an unknown Run")?;
        validate_run(run, revision.project_id.as_str())?;
        validate_provenance(provenance, &run.input_snapshot)?;
        validate_revision_lineage(request, revision, run)?;
    }
    Ok(())
}

fn validate_attempts(
    request: &RestorePackageRequest,
    run: &PackageRun,
    steps: &[&PackageRunStep],
) -> Result<(), String> {
    if request
        .run_steps
        .iter()
        .filter(|step| step.run_id == run.id)
        .count()
        != steps.len()
    {
        return Err("Package video translation Run contains another step kind".into());
    }
    let step_ids = steps
        .iter()
        .map(|step| step.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let attempts = request
        .run_attempts
        .iter()
        .filter(|attempt| {
            attempt.run_id == run.id || step_ids.contains(attempt.step_instance_id.as_str())
        })
        .collect::<Vec<_>>();
    if attempts.iter().any(|attempt| {
        attempt.run_id != run.id || !step_ids.contains(attempt.step_instance_id.as_str())
    }) || run.step_attempts != i64::try_from(attempts.len()).unwrap_or(-1)
    {
        return Err("Package video translation attempt ownership is invalid".into());
    }
    for step in steps {
        validate_step_attempts(
            step,
            attempts
                .iter()
                .copied()
                .filter(|attempt| attempt.step_instance_id == step.id),
        )?;
    }
    Ok(())
}

fn validate_step_attempts<'a>(
    step: &PackageRunStep,
    attempts: impl Iterator<Item = &'a PackageRunAttempt>,
) -> Result<(), String> {
    let mut numbers = std::collections::HashSet::new();
    let mut succeeded = 0_usize;
    for attempt in attempts {
        if attempt.attempt_number <= 0
            || !numbers.insert(attempt.attempt_number)
            || !matches!(
                attempt.state.as_str(),
                "succeeded" | "failed" | "cancelled" | "outcome_unknown"
            )
        {
            return Err("Package video translation attempts are incomplete".into());
        }
        if attempt.state == "succeeded" {
            succeeded += 1;
            if attempt.result.as_ref() != step.output.as_ref() {
                return Err(
                    "Package video translation attempt result differs from its step".into(),
                );
            }
        } else if attempt.result.is_some() {
            return Err("Package video translation terminal attempt contains a result".into());
        }
    }
    if succeeded != 1 {
        return Err("Package video translation step requires exactly one succeeded attempt".into());
    }
    Ok(())
}

fn validate_run(run: &PackageRun, project_id: &str) -> Result<(), String> {
    if run.state != "succeeded" || field(&run.input_snapshot, "projectId")? != project_id {
        return Err("Package video translation provenance does not match its succeeded Run".into());
    }
    Ok(())
}

fn validate_revision_lineage(
    request: &RestorePackageRequest,
    revision: &super::super::contracts::PackageSpeechProjectRevision,
    run: &PackageRun,
) -> Result<(), String> {
    let base_id = field(&run.input_snapshot, "revisionId")?;
    let base_number = integer(&run.input_snapshot, "expectedRevision")?;
    let base = request
        .speech_project_revisions
        .iter()
        .find(|candidate| candidate.id == base_id && candidate.project_id == revision.project_id)
        .ok_or("Package translated revision base is unavailable")?;
    if base.revision != base_number || revision.revision <= base_number {
        return Err("Package translated revision is not a successor of its Run base".into());
    }
    validate_run_base_sources(request, base, &run.input_snapshot)?;
    validate_revision_sources(request, &revision.id, &run.input_snapshot)?;
    if revision.revision == base_number + 1 {
        return Ok(());
    }
    let predecessor = request
        .speech_project_revisions
        .iter()
        .find(|candidate| {
            candidate.project_id == revision.project_id
                && candidate.revision == revision.revision - 1
        })
        .ok_or("Package translated revision predecessor is unavailable")?;
    let predecessor_run = predecessor
        .output_config
        .pointer("/translation/runId")
        .and_then(Value::as_str);
    if predecessor_run != Some(run.id.as_str()) {
        return Err("Package translated revision does not preserve predecessor provenance".into());
    }
    validate_revision_sources(request, &predecessor.id, &run.input_snapshot)
}

fn validate_run_base_sources(
    request: &RestorePackageRequest,
    base: &super::super::contracts::PackageSpeechProjectRevision,
    input: &Value,
) -> Result<(), String> {
    let sources = input
        .get("segments")
        .and_then(Value::as_array)
        .ok_or("Package video translation Run segments are unavailable")?;
    let mut segments = request
        .speech_segments
        .iter()
        .filter(|segment| segment.project_revision_id == base.id)
        .collect::<Vec<_>>();
    segments.sort_by_key(|segment| segment.ordinal);
    let translated = base
        .output_config
        .get("translation")
        .is_some_and(|value| !value.is_null());
    if segments.len() != sources.len() {
        return Err(
            "Package video translation Run source set differs from its base revision".into(),
        );
    }
    for (saved, source) in segments.iter().zip(sources) {
        let expected_id = saved.source_segment_id.as_deref().unwrap_or(&saved.id);
        let source_changed = !translated
            && saved
                .source_text
                .as_deref()
                .is_some_and(|text| text.trim() != saved.text.trim());
        let expected_revision = saved
            .source_segment_revision
            .unwrap_or(1)
            .checked_add(if source_changed { 1 } else { 0 })
            .ok_or("Package video source segment revision is exhausted")?;
        let expected_text = if translated {
            saved
                .source_text
                .as_deref()
                .ok_or("Package translated base segment source text is unavailable")?
        } else {
            &saved.text
        };
        if field(source, "sourceSegmentId")? != expected_id
            || integer(source, "sourceSegmentRevision")? != expected_revision
            || field(source, "text")?.trim() != expected_text.trim()
        {
            return Err(
                "Package video translation Run source snapshot differs from its base revision"
                    .into(),
            );
        }
    }
    Ok(())
}

fn validate_revision_sources(
    request: &RestorePackageRequest,
    revision_id: &str,
    input: &Value,
) -> Result<(), String> {
    let sources = input
        .get("segments")
        .and_then(Value::as_array)
        .ok_or("Package video translation Run segments are unavailable")?;
    let mut segments = request
        .speech_segments
        .iter()
        .filter(|segment| segment.project_revision_id == revision_id)
        .collect::<Vec<_>>();
    segments.sort_by_key(|segment| segment.ordinal);
    if segments.len() != sources.len()
        || segments.iter().zip(sources).any(|(saved, source)| {
            saved.source_segment_id.as_deref()
                != source.get("sourceSegmentId").and_then(Value::as_str)
                || saved.source_segment_revision
                    != source.get("sourceSegmentRevision").and_then(Value::as_i64)
                || saved.source_text.as_deref().map(str::trim)
                    != source.get("text").and_then(Value::as_str).map(str::trim)
        })
    {
        return Err("Package translated revision source identity differs from its Run".into());
    }
    Ok(())
}

fn validate_provenance(provenance: &Value, input: &Value) -> Result<(), String> {
    for field_name in ["targetLanguage", "provider", "model"] {
        if field(provenance, field_name)? != field(input, field_name)? {
            return Err(
                "Package video translation provenance differs from its Run snapshot".into(),
            );
        }
    }
    if provenance.get("localOnly").and_then(Value::as_bool)
        != input.get("localOnly").and_then(Value::as_bool)
    {
        return Err("Package video translation locality differs from its Run snapshot".into());
    }
    Ok(())
}

fn validate_steps(run: &PackageRun, steps: &[&PackageRunStep]) -> Result<(), String> {
    let segments = run
        .input_snapshot
        .get("segments")
        .and_then(Value::as_array)
        .ok_or("Package video translation Run segments are unavailable")?;
    if steps.len() != segments.len() {
        return Err("Package video translation checkpoints are incomplete".into());
    }
    let mut ordinals = std::collections::HashSet::with_capacity(steps.len());
    for step in steps {
        let output = step
            .output
            .as_ref()
            .ok_or("Package video translation checkpoint output is unavailable")?;
        let ordinal = step
            .configuration
            .get("ordinal")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .filter(|ordinal| *ordinal < segments.len())
            .ok_or("Package video translation checkpoint ordinal is invalid")?;
        let segment = step
            .configuration
            .get("segment")
            .ok_or("Package video translation checkpoint segment is unavailable")?;
        if !ordinals.insert(ordinal)
            || step.state != "succeeded"
            || field(output, "stepId")? != step.id
            || field(&step.configuration, "projectId")? != field(&run.input_snapshot, "projectId")?
            || field(&step.configuration, "revisionId")?
                != field(&run.input_snapshot, "revisionId")?
            || integer(&step.configuration, "expectedRevision")?
                != integer(&run.input_snapshot, "expectedRevision")?
            || field(&step.configuration, "inputRevision")?
                != field(&run.input_snapshot, "inputRevision")?
            || field(&step.configuration, "targetLanguage")?
                != field(&run.input_snapshot, "targetLanguage")?
            || field(&step.configuration, "provider")? != field(&run.input_snapshot, "provider")?
            || field(&step.configuration, "model")? != field(&run.input_snapshot, "model")?
            || field(&step.configuration, "baseUrl")? != field(&run.input_snapshot, "baseUrl")?
            || step.configuration.get("localOnly") != run.input_snapshot.get("localOnly")
            || step.configuration.get("executionTarget")
                != run.input_snapshot.get("executionTarget")
            || segment != &segments[ordinal]
            || output.get("sourceSegmentId") != segment.get("sourceSegmentId")
            || output.get("sourceSegmentRevision") != segment.get("sourceSegmentRevision")
            || output.get("sourceText") != segment.get("text")
        {
            return Err("Package video translation checkpoint identity is invalid".into());
        }
    }
    Ok(())
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Package video translation {name} is unavailable"))
}

fn integer(value: &Value, name: &str) -> Result<i64, String> {
    value
        .get(name)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("Package video translation {name} is unavailable"))
}
