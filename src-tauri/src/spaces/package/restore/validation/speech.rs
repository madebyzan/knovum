use std::collections::{HashMap, HashSet};

use super::super::contracts::RestorePackageRequest;

pub(super) fn validate(request: &RestorePackageRequest) -> Result<(), String> {
    validate_runtime_snapshots(request)?;
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
        request
            .speech_source_separations
            .iter()
            .map(|row| row.id.as_str()),
        "speech source separation",
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
    let separation_rows = request
        .speech_source_separations
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let run_ids = request
        .runs
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    ids(
        request.voice_candidates.iter().map(|row| row.id.as_str()),
        "voice candidate",
    )?;
    for row in &request.voice_candidates {
        reference(&run_ids, &row.run_id, "voice candidate run")?;
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
    for row in &request.speech_source_separations {
        reference(&run_ids, &row.run_id, "source separation run")?;
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
            crate::spaces::speech::preset_voices::require(
                row.preset_voice_id
                    .as_deref()
                    .ok_or("Package preset speaker is unavailable")?,
                &row.language,
            )?;
            let config =
                crate::spaces::speech::contracts::parse_synthesis_config(&row.synthesis_config)?;
            if config.sample_rate != 24_000 {
                return Err("Kokoro voices require 24 kHz synthesis".into());
            }
        } else {
            let config =
                crate::spaces::speech::contracts::parse_synthesis_config(&row.synthesis_config)?;
            if row.engine_id != "voxcpm2"
                || row.model_spec_id != crate::spaces::speech::voice_candidates::MODEL_SPEC_ID
                || config.sample_rate != 48_000
                || config.speed != 1.0
                || row
                    .authorization
                    .get("declared")
                    .and_then(serde_json::Value::as_bool)
                    != Some(true)
                || (row.reference_text.is_some()
                    && row
                        .authorization
                        .get("referenceTranscriptConfirmed")
                        .and_then(serde_json::Value::as_bool)
                        != Some(true))
            {
                return Err(
                    "Package reference voice differs from its authorized VoxCPM2 contract".into(),
                );
            }
        }
    }

    let lexicons = request
        .speech_project_revisions
        .iter()
        .map(|revision| {
            crate::spaces::speech::text::parse_lexicon(&revision.pronunciation_lexicon)
                .map(|lexicon| (revision.id.as_str(), lexicon))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    let mut chapter_segments: HashMap<&str, Vec<&super::super::contracts::PackageSpeechSegment>> =
        HashMap::new();
    for segment in &request.speech_segments {
        chapter_segments
            .entry(&segment.project_revision_id)
            .or_default()
            .push(segment);
    }
    for segments in chapter_segments.values_mut() {
        segments.sort_by_key(|segment| segment.ordinal);
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
        let project_type = project_types
            .get(row.project_id.as_str())
            .ok_or("Package speech revision owner is unavailable")?;
        crate::spaces::speech::project_config::validate(project_type, &row.output_config)?;
        if *project_type == "audiobook" {
            let config =
                crate::spaces::speech::project_config::parse_audiobook(&row.output_config)?;
            for role in &config.roles {
                reference(&revisions, &role.voice_revision_id, "audiobook role voice")?;
            }
            let ordered = chapter_segments
                .get(row.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            crate::spaces::speech::project_config::validate_chapter_order(
                &config,
                ordered.iter().map(|segment| segment.chapter_key.as_deref()),
            )?;
            for segment in ordered {
                let role_voice = crate::spaces::speech::project_config::role_voice(
                    &config,
                    segment.speaker_key.as_deref(),
                )?;
                if role_voice != segment.voice_revision_id {
                    return Err(
                        "Package audiobook paragraph voice differs from its role binding".into(),
                    );
                }
            }
            if ordered
                .iter()
                .any(|segment| segment.source_segment_id.is_some())
            {
                return Err("Package audiobook contains video source metadata".into());
            }
        } else if *project_type == "video_dubbing" {
            let config = crate::spaces::speech::video_config::parse(&row.output_config)?;
            if let Some(provenance) = &config.background_separation {
                let separation = separation_rows
                    .get(provenance.separation_id.as_str())
                    .ok_or("Package video references an unavailable source separation")?;
                let background = config
                    .background
                    .as_ref()
                    .ok_or("Package separated video background is unavailable")?;
                if separation.review_status != "accepted"
                    || separation.model_spec_id != provenance.model_spec_id
                    || separation.source_asset_path != provenance.source.asset_path
                    || separation.source_revision != provenance.source.source_revision
                    || separation.source_duration_ms != provenance.source.duration_ms as i64
                    || separation.background_asset_path != background.asset_path
                    || format!("sha256:{}", separation.background_sha256)
                        != background.source_revision
                    || separation.background_duration_ms != background.duration_ms as i64
                {
                    return Err("Package video source separation provenance does not match its accepted candidate".into());
                }
            }
            let ordered = chapter_segments
                .get(row.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            for segment in ordered {
                crate::spaces::speech::video_config::validate_timing(
                    &config,
                    segment.target_start_ms,
                    segment.target_end_ms,
                    segment.chapter_key.as_deref(),
                )?;
            }
            if config.translation.is_some()
                && ordered
                    .iter()
                    .any(|segment| segment.source_segment_id.is_none())
            {
                return Err(
                    "Translated package video revision requires source identity for every segment"
                        .into(),
                );
            }
        } else if chapter_segments
            .get(row.id.as_str())
            .is_some_and(|segments| {
                segments.iter().any(|segment| {
                    segment.chapter_key.is_some()
                        || segment.target_start_ms.is_some()
                        || segment.target_end_ms.is_some()
                        || segment.source_segment_id.is_some()
                })
            })
        {
            return Err("Package short dubbing contains chapter or video constraints".into());
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
        let pronunciations = lexicons
            .get(row.project_revision_id.as_str())
            .ok_or("Package speech segment revision is unavailable")?;
        if row.normalized_text
            != crate::spaces::speech::text::apply_lexicon(&row.text, pronunciations)
        {
            return Err(
                "Package speech segment normalized text differs from its source and lexicon".into(),
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
        match (
            row.source_segment_id.as_deref(),
            row.source_segment_revision,
            row.source_text.as_deref(),
        ) {
            (None, None, None) => {}
            (Some(id), Some(revision), Some(text))
                if !id.trim().is_empty()
                    && id.encode_utf16().count() <= 240
                    && revision > 0
                    && !text.trim().is_empty()
                    && text.chars().count() <= 16_000 => {}
            _ => return Err("Package speech segment source identity is invalid".into()),
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

fn validate_runtime_snapshots(request: &RestorePackageRequest) -> Result<(), String> {
    for run in &request.runs {
        if run
            .input_snapshot
            .get("workflow")
            .and_then(serde_json::Value::as_str)
            != Some("speech_project_generation_v1")
        {
            continue;
        }
        let digest = run
            .input_snapshot
            .get("runtimeDigest")
            .and_then(serde_json::Value::as_str)
            .ok_or("Package speech Run has no frozen Python runtime digest")?;
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err("Package speech Run Python runtime digest is invalid".into());
        }
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
