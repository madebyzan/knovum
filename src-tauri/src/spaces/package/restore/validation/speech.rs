use std::collections::{HashMap, HashSet};

use super::super::contracts::RestorePackageRequest;

pub(super) fn validate(request: &RestorePackageRequest) -> Result<(), String> {
    let profiles = ids(
        request.voice_profiles.iter().map(|row| row.id.as_str()),
        "voice profile",
    )?;
    let revisions = ids(
        request.voice_revisions.iter().map(|row| row.id.as_str()),
        "voice revision",
    )?;
    let projects = ids(
        request.speech_projects.iter().map(|row| row.id.as_str()),
        "speech project",
    )?;
    let project_revisions = ids(
        request
            .speech_project_revisions
            .iter()
            .map(|row| row.id.as_str()),
        "speech project revision",
    )?;
    let segments = ids(
        request.speech_segments.iter().map(|row| row.id.as_str()),
        "speech segment",
    )?;
    ids(
        request.speech_renders.iter().map(|row| row.id.as_str()),
        "speech render",
    )?;

    let revision_owners = request
        .voice_revisions
        .iter()
        .map(|row| (row.id.as_str(), row.profile_id.as_str()))
        .collect::<HashMap<_, _>>();
    let voice_languages = request
        .voice_revisions
        .iter()
        .map(|row| (row.id.as_str(), row.language.as_str()))
        .collect::<HashMap<_, _>>();
    let project_types = request
        .speech_projects
        .iter()
        .map(|row| (row.id.as_str(), row.project_type.as_str()))
        .collect::<HashMap<_, _>>();
    let project_revision_owners = request
        .speech_project_revisions
        .iter()
        .map(|row| (row.id.as_str(), row.project_id.as_str()))
        .collect::<HashMap<_, _>>();
    let mut voice_versions = HashSet::new();
    for row in &request.voice_profiles {
        required(&row.name, 120, "Voice profile name")?;
        if !matches!(
            row.source_kind.as_str(),
            "preset" | "reference" | "designed"
        ) {
            return Err("Package voice profile source kind is invalid".into());
        }
        if row.created_at < 0 || row.updated_at < row.created_at {
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
        reference(&profiles, &row.profile_id, "voice revision profile")?;
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
            if row.engine_id != "kokoro"
                || row.model_spec_id != crate::spaces::speech::execution::KOKORO_MODEL_SPEC_ID
                || row
                    .preset_voice_id
                    .as_deref()
                    .and_then(|value| value.parse::<u32>().ok())
                    .is_none()
            {
                return Err(
                    "Package preset voice does not match the supported Kokoro contract".into(),
                );
            }
            crate::spaces::speech::contracts::parse_synthesis_config(&row.synthesis_config)?;
        }
    }

    let mut project_versions = HashSet::new();
    for row in &request.speech_projects {
        required(&row.title, 200, "Speech project title")?;
        if !matches!(
            row.project_type.as_str(),
            "short_dubbing" | "audiobook" | "video_dubbing"
        ) {
            return Err("Package speech project type is invalid".into());
        }
        if row.created_at < 0 || row.updated_at < row.created_at {
            return Err("Package speech project timestamps are invalid".into());
        }
        let active = row
            .active_revision_id
            .as_deref()
            .ok_or("Package speech project requires an active revision")?;
        if project_revision_owners.get(active) != Some(&row.id.as_str()) {
            return Err("Package speech project active revision has a different owner".into());
        }
    }
    for row in &request.speech_project_revisions {
        reference(&projects, &row.project_id, "speech project revision owner")?;
        if row.revision <= 0 || !project_versions.insert((row.project_id.as_str(), row.revision)) {
            return Err(
                "Package speech project revision numbers must be positive and unique per project"
                    .into(),
            );
        }
        required(&row.source_revision, usize::MAX, "Speech source revision")?;
        if !row.pronunciation_lexicon.is_object()
            || !row.output_config.is_object()
            || row.created_at < 0
        {
            return Err("Package speech project revision metadata is invalid".into());
        }
        if project_types.get(row.project_id.as_str()) == Some(&"short_dubbing")
            && row
                .output_config
                .get("format")
                .and_then(serde_json::Value::as_str)
                != Some("wav")
        {
            return Err("Package short dubbing revision requires WAV output".into());
        }
        validate_project_output(row)?;
    }

    let mut ordinals = HashSet::new();
    for row in &request.speech_segments {
        reference(
            &project_revisions,
            &row.project_revision_id,
            "speech segment revision",
        )?;
        reference(&revisions, &row.voice_revision_id, "speech segment voice")?;
        if row.ordinal < 0 || !ordinals.insert((row.project_revision_id.as_str(), row.ordinal)) {
            return Err(
                "Package speech segment ordinals must be nonnegative and unique per revision"
                    .into(),
            );
        }
        required(&row.text, usize::MAX, "Speech segment text")?;
        required(&row.normalized_text, usize::MAX, "Speech normalized text")?;
        if row.normalized_text != crate::spaces::speech::text::normalize(&row.text) {
            return Err(
                "Package speech segment normalized text differs from its source text".into(),
            );
        }
        required(&row.language, usize::MAX, "Speech segment language")?;
        if voice_languages.get(row.voice_revision_id.as_str()) != Some(&row.language.as_str()) {
            return Err("Package speech segment language differs from its voice revision".into());
        }
        if !matches!(
            row.state.as_str(),
            "pending" | "ready" | "rendered" | "needs_edit" | "failed"
        ) {
            return Err("Package speech segment state is not stable".into());
        }
        match (row.target_start_ms, row.target_end_ms) {
            (None, None) => {}
            (Some(start), Some(end)) if start >= 0 && end > start => {}
            _ => return Err("Package speech segment timing is invalid".into()),
        }
        if row.created_at < 0 || row.updated_at < row.created_at {
            return Err("Package speech segment timestamps are invalid".into());
        }
    }

    let mut attempts = HashSet::new();
    for row in &request.speech_renders {
        reference(&segments, &row.segment_id, "speech render segment")?;
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

fn validate_voice_source(
    row: &super::super::contracts::PackageVoiceRevision,
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
                    .is_some_and(|value| !value.trim().is_empty())
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
                && row.reference_asset_id.is_none()
                && row.reference_source_revision.is_none()
                && row.reference_text.is_none()
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

fn validate_render(row: &super::super::contracts::PackageSpeechRender) -> Result<(), String> {
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

fn validate_project_output(
    row: &super::super::contracts::PackageSpeechProjectRevision,
) -> Result<(), String> {
    match (
        &row.output_asset_path,
        &row.output_sha256,
        row.output_sample_rate,
        row.output_sample_count,
        row.output_published_at,
    ) {
        (None, None, None, None, None) => Ok(()),
        (Some(path), Some(digest), Some(rate), Some(samples), Some(published))
            if !path.is_empty() && rate > 0 && samples >= 0 && published >= row.created_at =>
        {
            hex64(digest, "Speech project output digest")
        }
        _ => Err("Package speech project output metadata is incomplete".into()),
    }
}

fn ids<'a>(values: impl Iterator<Item = &'a str>, label: &str) -> Result<HashSet<&'a str>, String> {
    let mut result = HashSet::new();
    for value in values {
        if value.is_empty() || !result.insert(value) {
            return Err(format!("Package {label} IDs must be nonempty and unique"));
        }
    }
    Ok(result)
}

fn reference(ids: &HashSet<&str>, id: &str, label: &str) -> Result<(), String> {
    if ids.contains(id) {
        Ok(())
    } else {
        Err(format!("Package {label} reference is unavailable"))
    }
}

fn required(value: &str, max: usize, label: &str) -> Result<(), String> {
    let count = value.trim().chars().count();
    if count == 0 || count > max {
        Err(format!("{label} is invalid"))
    } else {
        Ok(())
    }
}

fn hex64(value: &str, label: &str) -> Result<(), String> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!("{label} must be a 64-character hexadecimal digest"))
    }
}

fn valid_revision(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
