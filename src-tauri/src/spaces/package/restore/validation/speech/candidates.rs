use super::{hex64, reference, valid_revision, SpeechIndex};
use crate::spaces::package::restore::contracts::RestorePackageRequest;

pub(super) fn validate(
    request: &RestorePackageRequest,
    index: &SpeechIndex<'_>,
) -> Result<(), String> {
    let SpeechIndex {
        profiles,
        revisions,
        revision_owners,
        run_ids,
        ..
    } = index;
    for row in &request.voice_candidates {
        reference(run_ids, &row.run_id, "voice candidate run")?;
        if row.id != row.run_id
            || !matches!(row.source_kind.as_str(), "reference" | "designed")
            || !matches!(row.language.as_str(), "zh-CN" | "en")
            || row.sample_text.trim().is_empty()
            || row.sample_text.chars().count() > 500
            || row.reference_asset_path.trim().is_empty()
            || row.preview_asset_path.trim().is_empty()
            || row.reference_byte_length <= 44
            || row.preview_sample_rate <= 0
            || row.preview_sample_count <= 0
            || row.model_spec_id != crate::spaces::speech::voice_candidates::MODEL_SPEC_ID
            || !row.synthesis_config.is_object()
            || row
                .authorization
                .get("declared")
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            || row.created_at < 0
        {
            return Err("Package voice candidate metadata is invalid".into());
        }
        hex64(&row.reference_sha256, "Voice candidate reference digest")?;
        hex64(&row.preview_sha256, "Voice candidate preview digest")?;
        match row.source_kind.as_str() {
            "designed"
                if row
                    .design_description
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                    && row.reference_text.is_some()
                    && row.input_source_revision.is_none()
                    && row.input_byte_length.is_none() => {}
            "reference"
                if row.design_description.is_none()
                    && row
                        .input_source_revision
                        .as_deref()
                        .is_some_and(valid_revision)
                    && row
                        .input_byte_length
                        .is_some_and(|value| value > 0 && value <= 128 * 1024 * 1024) => {}
            _ => return Err("Package voice candidate source metadata is invalid".into()),
        }
        match (
            row.status.as_str(),
            row.auditioned_at,
            row.selected_voice_profile_id.as_deref(),
            row.selected_voice_revision_id.as_deref(),
            row.selected_at,
            row.rejected_at,
        ) {
            ("ready", auditioned, None, None, None, None)
                if auditioned.is_none_or(|value| value >= row.created_at) => {}
            ("selected", Some(auditioned), Some(profile), Some(revision), Some(selected), None)
                if auditioned >= row.created_at
                    && selected >= auditioned
                    && profiles.contains(profile)
                    && revisions.contains(revision)
                    && revision_owners.get(revision) == Some(&profile) => {}
            ("rejected", auditioned, None, None, None, Some(rejected))
                if rejected >= row.created_at
                    && auditioned
                        .is_none_or(|value| value >= row.created_at && value <= rejected) => {}
            _ => return Err("Package voice candidate selection state is invalid".into()),
        }
    }
    Ok(())
}
