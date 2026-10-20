use super::{hex64, reference, valid_revision, SpeechIndex};
use crate::spaces::package::restore::contracts::RestorePackageRequest;

pub(super) fn validate(
    request: &RestorePackageRequest,
    index: &SpeechIndex<'_>,
) -> Result<(), String> {
    let SpeechIndex { run_ids, .. } = index;
    for row in &request.speech_source_separations {
        reference(run_ids, &row.run_id, "source separation run")?;
        let source = request
            .nodes
            .iter()
            .find(|node| node.id == row.source_node_id)
            .ok_or("Package source separation node is unavailable")?;
        if !matches!(source.kind.as_str(), "audio" | "video") {
            return Err("Package source separation requires an audio or video node".into());
        }

        let expected_duration_ms = row
            .background_sample_count
            .checked_mul(1_000)
            .and_then(|samples| samples.checked_add(44_099))
            .map(|samples| samples / 44_100);
        if row.id != row.run_id
            || crate::local_resources::model_catalog::LocalModelId::from_separation_spec(
                &row.model_spec_id,
            )
            .is_err()
            || row.source_duration_ms <= 0
            || row.source_duration_ms > 600_000
            || row.background_sample_rate != 44_100
            || row.background_sample_count <= 0
            || row.background_sample_count > 44_100 * 600
            || Some(row.background_duration_ms) != expected_duration_ms
            || row.source_asset_path.trim().is_empty()
            || row.background_asset_path.trim().is_empty()
            || row.vocals_asset_path.trim().is_empty()
            || row.vocals_asset_path == row.background_asset_path
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
        hex64(&row.vocals_sha256, "Source separation vocal digest")?;
    }
    Ok(())
}
