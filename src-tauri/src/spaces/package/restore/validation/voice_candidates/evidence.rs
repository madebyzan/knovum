use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{field, lower_hex64};
use crate::spaces::package::restore::contracts::{
    PackageRunAttempt, PackageRunStep, PackageVoiceCandidate,
};

pub(super) fn validate_metadata(candidate: &PackageVoiceCandidate) -> Result<(), String> {
    let config = candidate
        .synthesis_config
        .as_object()
        .ok_or("Package voice candidate synthesis configuration is invalid")?;
    let authorization = candidate
        .authorization
        .as_object()
        .ok_or("Package voice candidate authorization is invalid")?;
    if candidate.model_spec_id != crate::spaces::speech::voice_candidates::MODEL_SPEC_ID
        || candidate.seed != 0
        || config.len() != 2
        || config.get("speed").and_then(Value::as_f64) != Some(1.0)
        || config.get("sampleRate").and_then(Value::as_i64) != Some(48_000)
        || authorization.len() != 3
        || authorization.get("declared").and_then(Value::as_bool) != Some(true)
        || authorization.get("declaredAt").and_then(Value::as_i64) != Some(candidate.created_at)
        || authorization
            .get("referenceTranscriptConfirmed")
            .and_then(Value::as_bool)
            != Some(candidate.reference_text.is_some())
    {
        return Err("Package voice candidate execution metadata differs".into());
    }
    Ok(())
}

pub(super) fn validate_output(
    candidate: &PackageVoiceCandidate,
    output: &Value,
    space_id: &str,
) -> Result<(), String> {
    let audio = output
        .get("audio")
        .ok_or("Package voice candidate output audio is unavailable")?;
    if field(output, "candidateId")? != candidate.id
        || field(output, "runId")? != candidate.run_id
        || field(output, "spaceId")? != space_id
        || field(output, "sourceKind")? != candidate.source_kind
        || field(output, "language")? != candidate.language
        || field(output, "sampleText")? != candidate.sample_text
        || output.get("designDescription").and_then(Value::as_str)
            != candidate.design_description.as_deref()
        || output.get("referenceText").and_then(Value::as_str)
            != candidate.reference_text.as_deref()
        || field(audio, "sha256")? != candidate.preview_sha256
        || audio.get("sampleRate").and_then(Value::as_i64) != Some(candidate.preview_sample_rate)
        || audio.get("sampleCount").and_then(Value::as_i64) != Some(candidate.preview_sample_count)
        || audio.get("path").is_some()
        || field(output, "status")? != "ready"
        || output
            .get("auditionedAt")
            .is_none_or(|value| !value.is_null())
        || output
            .get("selectedVoiceProfileId")
            .is_none_or(|value| !value.is_null())
        || output
            .get("selectedVoiceRevisionId")
            .is_none_or(|value| !value.is_null())
        || output
            .get("selectedAt")
            .is_none_or(|value| !value.is_null())
        || output
            .get("rejectedAt")
            .is_none_or(|value| !value.is_null())
        || output.get("createdAt").and_then(Value::as_i64) != Some(candidate.created_at)
    {
        return Err("Package voice candidate output differs from its candidate".into());
    }
    Ok(())
}

pub(super) fn validate_attempts(
    step: &PackageRunStep,
    attempts: &[&PackageRunAttempt],
    output: &Value,
) -> Result<(), String> {
    let encoded = serde_json::to_string(&step.resolved_input).map_err(|error| error.to_string())?;
    let digest = format!("{:x}", Sha256::digest(encoded.as_bytes()));
    for attempt in attempts {
        let expected_result = (attempt.state == "succeeded").then_some(output);
        if !lower_hex64(&attempt.request_digest)
            || attempt.request_digest != digest
            || attempt.idempotency_key != step.id
            || attempt.result.as_ref() != expected_result
            || attempt
                .finished_at
                .is_none_or(|finished| finished < attempt.started_at)
        {
            return Err("Package voice candidate attempt provenance is invalid".into());
        }
    }
    Ok(())
}
