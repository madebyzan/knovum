use super::{hex64, reference, valid_revision, SpeechIndex};
use crate::spaces::package::restore::contracts::RestorePackageRequest;

pub(super) fn validate(
    request: &RestorePackageRequest,
    index: &SpeechIndex<'_>,
) -> Result<(), String> {
    let SpeechIndex { run_ids, .. } = index;
    for row in &request.speech_source_separations {
        reference(run_ids, &row.run_id, "source separation run")?;
        let expected_duration = row
            .background_sample_count
            .checked_mul(1_000)
            .and_then(|samples| samples.checked_add(44_099))
            .map(|samples| samples / 44_100);
        if row.id != row.run_id
            || row.model_spec_id != crate::spaces::speech::source_separation::MODEL_SPEC_ID
            || row.source_duration_ms <= 0
            || row.source_duration_ms > 600_000
            || row.background_sample_rate != 44_100
            || row.background_sample_count <= 0
            || row.background_sample_count > 44_100 * 600
            || Some(row.background_duration_ms) != expected_duration
            || row.source_asset_path.trim().is_empty()
            || row.background_asset_path.trim().is_empty()
            || row.review_warning != crate::spaces::speech::source_separation::review_warning()
            || row.created_at < 0
        {
            return Err("Package source separation metadata is invalid".into());
        }
        if !valid_revision(&row.source_revision) {
            return Err("Source separation revision is invalid".into());
        }
        hex64(
            &row.background_sha256,
            "Source separation background digest",
        )?;
        match (row.review_status.as_str(), row.reviewed_at) {
            ("pending", None) => {}
            ("accepted" | "rejected", Some(reviewed)) if reviewed >= row.created_at => {}
            _ => return Err("Package source separation review state is invalid".into()),
        }
    }
    Ok(())
}
