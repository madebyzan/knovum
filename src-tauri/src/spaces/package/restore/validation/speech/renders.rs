use super::{hex64, reference, required, SpeechIndex};
use crate::spaces::package::restore::contracts::RestorePackageRequest;
use std::collections::HashSet;

pub(super) fn validate(
    request: &RestorePackageRequest,
    index: &SpeechIndex<'_>,
) -> Result<(), String> {
    let SpeechIndex { segments, .. } = index;
    let mut attempts = HashSet::new();
    for row in &request.speech_renders {
        reference(segments, &row.segment_id, "speech render segment")?;
        if row.attempt_epoch < 0 || !attempts.insert((row.segment_id.as_str(), row.attempt_epoch)) {
            return Err(
                "Package speech render attempts must be nonnegative and unique per segment".into(),
            );
        }
        hex64(&row.input_digest, "Speech render input digest")?;
        required(&row.model_spec_id, usize::MAX, "Speech render model")?;
        required(&row.engine_version, usize::MAX, "Speech render engine")?;
        if !row.synthesis_config.is_object() || row.created_at < 0 {
            return Err("Package speech render metadata is invalid".into());
        }
        validate_render(row)?;
    }
    Ok(())
}

fn validate_render(
    row: &crate::spaces::package::restore::contracts::PackageSpeechRender,
) -> Result<(), String> {
    if row.state == "published" {
        let digest = row
            .output_sha256
            .as_deref()
            .ok_or("Published speech render requires an output digest")?;
        hex64(digest, "Speech render output digest")?;
        if row.asset_path.as_deref().is_none_or(str::is_empty)
            || row.sample_rate.is_none_or(|value| value <= 0)
            || row.sample_count.is_none_or(|value| value < 0)
            || row.published_at.is_none_or(|value| value < row.created_at)
        {
            return Err("Published speech render metadata is incomplete".into());
        }
        return Ok(());
    }
    if !matches!(row.state.as_str(), "rejected" | "failed" | "cancelled") {
        return Err("Package speech render state is not stable".into());
    }
    if row.asset_path.is_some()
        || row.output_sha256.is_some()
        || row.sample_rate.is_some()
        || row.sample_count.is_some()
        || row.published_at.is_some()
    {
        return Err("Unpublished speech render must not retain a published asset".into());
    }
    Ok(())
}
