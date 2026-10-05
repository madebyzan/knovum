use super::{evidence, field};
use crate::spaces::package::restore::contracts::{
    PackageRun, PackageRunAttempt, PackageRunStep, PackageVoiceCandidate, RestorePackageRequest,
};
use serde_json::Value;

pub(super) fn validate(
    request: &RestorePackageRequest,
    candidate: &PackageVoiceCandidate,
    run: &PackageRun,
    step: &PackageRunStep,
    attempts: &[&PackageRunAttempt],
) -> Result<(), String> {
    let configuration = &step.configuration;
    let draft = configuration
        .get("request")
        .ok_or("Package voice candidate request snapshot is unavailable")?;
    if configuration.as_object().map(serde_json::Map::len) != Some(2)
        || draft.as_object().map(serde_json::Map::len) != Some(7)
    {
        return Err("Package voice candidate configuration shape is invalid".into());
    }
    validate_draft(candidate, run, draft, &request.space.id)?;
    validate_source(candidate, run, configuration, draft)?;
    evidence::validate_metadata(candidate)?;
    let output = step
        .output
        .as_ref()
        .ok_or("Package voice candidate output is unavailable")?;
    evidence::validate_output(candidate, output, &request.space.id)?;
    evidence::validate_attempts(step, attempts, output)?;
    if candidate.status == "selected" {
        validate_selected_voice(request, candidate)?;
    }
    Ok(())
}

fn validate_draft(
    candidate: &PackageVoiceCandidate,
    run: &PackageRun,
    draft: &Value,
    space_id: &str,
) -> Result<(), String> {
    if field(draft, "requestId")? != candidate.id
        || field(draft, "spaceId")? != space_id
        || field(draft, "language")? != candidate.language
        || field(draft, "text")?.trim() != candidate.sample_text
        || draft.get("authorized").and_then(Value::as_bool) != Some(true)
        || draft.get("seed").and_then(Value::as_i64) != Some(candidate.seed)
        || run.id != candidate.id
    {
        return Err("Package voice candidate differs from its request snapshot".into());
    }
    Ok(())
}

fn validate_source(
    candidate: &PackageVoiceCandidate,
    run: &PackageRun,
    configuration: &Value,
    draft: &Value,
) -> Result<(), String> {
    let source = draft
        .get("source")
        .ok_or("Package voice candidate source snapshot is unavailable")?;
    let run_source = run.input_snapshot.get("referenceSource");
    let step_source = configuration.get("referenceSource");
    if run_source != step_source {
        return Err("Package voice candidate frozen source snapshots differ".into());
    }
    match candidate.source_kind.as_str() {
        "designed"
            if source.as_object().map(serde_json::Map::len) == Some(2)
                && field(source, "kind")? == "designed"
                && source
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    == candidate.design_description.as_deref()
                && run_source.is_some_and(Value::is_null) => {}
        "reference"
            if source.as_object().map(serde_json::Map::len) == Some(7)
                && field(source, "kind")? == "reference"
                && source.get("sourcePath").is_some_and(Value::is_null)
                && source
                    .get("referenceText")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    == candidate.reference_text.as_deref()
                && source
                    .get("referenceTextConfirmed")
                    .and_then(Value::as_bool)
                    .is_some_and(|confirmed| candidate.reference_text.is_none() || confirmed) =>
        {
            let start = source
                .get("startMs")
                .and_then(Value::as_u64)
                .ok_or("Package reference selection start is unavailable")?;
            let end = source
                .get("endMs")
                .and_then(Value::as_u64)
                .ok_or("Package reference selection end is unavailable")?;
            if end <= start
                || !(5000..=30000).contains(&(end - start))
                || end > 43_200_000
                || source.get("sourceRevision").and_then(Value::as_str)
                    != candidate.input_source_revision.as_deref()
            {
                return Err("Package reference selection differs from its frozen source".into());
            }
            let snapshot = run_source
                .filter(|value| value.is_object())
                .ok_or("Package voice candidate frozen source is unavailable")?;
            if snapshot.as_object().map(serde_json::Map::len) != Some(2)
                || snapshot.get("assetPath").is_some()
                || field(snapshot, "sourceRevision")?
                    != candidate
                        .input_source_revision
                        .as_deref()
                        .unwrap_or_default()
                || snapshot.get("byteLength").and_then(Value::as_i64) != candidate.input_byte_length
            {
                return Err("Package voice candidate source revision differs".into());
            }
        }
        _ => return Err("Package voice candidate source provenance is invalid".into()),
    }
    Ok(())
}

fn validate_selected_voice(
    request: &RestorePackageRequest,
    candidate: &PackageVoiceCandidate,
) -> Result<(), String> {
    let profile_id = candidate
        .selected_voice_profile_id
        .as_deref()
        .ok_or("Selected package voice candidate profile is unavailable")?;
    let revision_id = candidate
        .selected_voice_revision_id
        .as_deref()
        .ok_or("Selected package voice candidate revision is unavailable")?;
    let profile = request
        .voice_profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or("Selected package voice candidate profile is unavailable")?;
    let revision = request
        .voice_revisions
        .iter()
        .find(|revision| revision.id == revision_id && revision.profile_id == profile.id)
        .ok_or("Selected package voice candidate revision is unavailable")?;
    let authorization = revision
        .authorization
        .as_object()
        .ok_or("Selected package voice revision authorization is invalid")?;
    let reference_revision = format!("sha256:{}", candidate.reference_sha256);
    if profile.source_kind != candidate.source_kind
        || revision.engine_id != "voxcpm2"
        || revision.model_spec_id != candidate.model_spec_id
        || revision.language != candidate.language
        || revision.reference_source_revision.as_deref() != Some(reference_revision.as_str())
        || revision.reference_text.as_deref() != candidate.reference_text.as_deref()
        || revision.design_description.as_deref() != candidate.design_description.as_deref()
        || revision.synthesis_config != candidate.synthesis_config
        || authorization.get("declared").and_then(Value::as_bool) != Some(true)
        || authorization
            .get("referenceTranscriptConfirmed")
            .and_then(Value::as_bool)
            != Some(candidate.reference_text.is_some())
        || authorization.get("candidateId").and_then(Value::as_str) != Some(candidate.id.as_str())
        || authorization.get("candidateRunId").and_then(Value::as_str)
            != Some(candidate.run_id.as_str())
    {
        return Err("Selected package voice revision differs from its candidate".into());
    }
    Ok(())
}
