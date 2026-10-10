use super::{reference, required, valid_revision, SpeechIndex};
use crate::spaces::package::restore::contracts::RestorePackageRequest;
use std::collections::HashSet;

pub(super) fn validate(
    request: &RestorePackageRequest,
    index: &SpeechIndex<'_>,
) -> Result<(), String> {
    let SpeechIndex {
        profiles,
        revision_owners,
        ..
    } = index;
    let mut voice_versions = HashSet::new();
    for row in &request.voice_profiles {
        required(&row.name, 120, "Voice profile name")?;
        if !matches!(
            row.source_kind.as_str(),
            "preset" | "reference" | "designed"
        ) {
            return Err("Package voice profile source kind is invalid".into());
        }
        if row.created_at < 0
            || row.updated_at < row.created_at
            || row
                .archived_at
                .is_some_and(|archived| archived < row.created_at || archived > row.updated_at)
        {
            return Err("Package voice profile timestamps are invalid".into());
        }
        let active = row
            .active_revision_id
            .as_deref()
            .ok_or("Package voice profile requires an active revision")?;
        if revision_owners.get(active) != Some(&row.id.as_str()) {
            return Err("Package voice profile active revision has a different owner".into());
        }
    }
    for row in &request.voice_revisions {
        reference(profiles, &row.profile_id, "voice revision profile")?;
        if row.revision <= 0 || !voice_versions.insert((row.profile_id.as_str(), row.revision)) {
            return Err(
                "Package voice revision numbers must be positive and unique per profile".into(),
            );
        }
        required(&row.engine_id, usize::MAX, "Voice engine")?;
        required(&row.model_spec_id, usize::MAX, "Voice model specification")?;
        required(&row.language, usize::MAX, "Voice language")?;
        if !row.synthesis_config.is_object() || !row.authorization.is_object() || row.created_at < 0
        {
            return Err("Package voice revision metadata is invalid".into());
        }
        validate_voice_source(row, request)?;
        let source_kind = request
            .voice_profiles
            .iter()
            .find(|profile| profile.id == row.profile_id)
            .map(|profile| profile.source_kind.as_str())
            .ok_or("Package voice profile is unavailable")?;
        if source_kind == "preset" {
            if row.engine_id != "moss_nano"
                || row.model_spec_id != crate::spaces::speech::execution::MOSS_NANO_MODEL_SPEC_ID
                || row
                    .preset_voice_id
                    .as_deref()
                    .and_then(|value| value.parse::<u32>().ok())
                    .is_none()
            {
                return Err(
                    "Package preset voice does not match the supported MossNano contract".into(),
                );
            }
            crate::spaces::speech::preset_voices::require(
                row.preset_voice_id
                    .as_deref()
                    .ok_or("Package preset speaker is unavailable")?,
                &row.language,
            )?;
            let config =
                crate::spaces::speech::contracts::parse_synthesis_config(&row.synthesis_config)?;
            if config.sample_rate != 48_000 {
                return Err("MossNano voices require 48 kHz synthesis".into());
            }
        } else {
            let config =
                crate::spaces::speech::contracts::parse_synthesis_config(&row.synthesis_config)?;
            let model = crate::spaces::speech::voice_candidates::VoiceModel::from_specification(
                &row.model_spec_id,
            )?;
            if row.engine_id != model.engine()
                || (model == crate::spaces::speech::voice_candidates::VoiceModel::MossNano
                    && (source_kind != "reference"
                        || row.reference_text.is_some()
                        || row.design_description.is_some()))
                || config.sample_rate != 48_000
                || config.speed != 1.0
                || (row.reference_text.is_some()
                    && row
                        .authorization
                        .get("referenceTranscriptConfirmed")
                        .and_then(serde_json::Value::as_bool)
                        != Some(true))
            {
                return Err(
                    "Package reference voice differs from its synthesis model contract".into(),
                );
            }
        }
    }

    Ok(())
}

fn validate_voice_source(
    row: &crate::spaces::package::restore::contracts::PackageVoiceRevision,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    let source_kind = request
        .voice_profiles
        .iter()
        .find(|profile| profile.id == row.profile_id)
        .map(|profile| profile.source_kind.as_str())
        .ok_or("Package voice profile is unavailable")?;
    match source_kind {
        "preset"
            if row
                .preset_voice_id
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
                && row.reference_asset_id.is_none()
                && row.reference_source_revision.is_none()
                && row.reference_text.is_none()
                && row.design_description.is_none() =>
        {
            Ok(())
        }
        "reference"
            if row.preset_voice_id.is_none()
                && row.reference_asset_id.is_some()
                && row
                    .reference_text
                    .as_deref()
                    .is_none_or(|value| !value.trim().is_empty())
                && row.design_description.is_none() =>
        {
            let revision = row
                .reference_source_revision
                .as_deref()
                .ok_or("Package reference voice requires a source revision")?;
            if valid_revision(revision) {
                Ok(())
            } else {
                Err("Package reference voice source revision is invalid".into())
            }
        }
        "designed"
            if row.preset_voice_id.is_none()
                && row.reference_asset_id.is_some()
                && row
                    .reference_source_revision
                    .as_deref()
                    .is_some_and(valid_revision)
                && row
                    .reference_text
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                && row
                    .design_description
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty()) =>
        {
            Ok(())
        }
        _ => Err("Package voice revision source does not match its profile".into()),
    }
}
