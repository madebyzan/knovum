use super::{hex64, reference, required, SpeechIndex};
use crate::spaces::package::restore::contracts::RestorePackageRequest;
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    request: &RestorePackageRequest,
    index: &SpeechIndex<'_>,
) -> Result<(), String> {
    let SpeechIndex {
        revisions,
        projects,
        project_types,
        project_revision_owners,
        separation_rows,
        ..
    } = index;
    let mut chapter_segments: HashMap<
        &str,
        Vec<&crate::spaces::package::restore::contracts::PackageSpeechSegment>,
    > = HashMap::new();
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
    let nodes: HashMap<_, _> = request
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    for row in &request.speech_projects {
        let source = nodes
            .get(row.source_node_id.as_str())
            .ok_or("Package speech source node is unavailable")?;
        crate::spaces::speech::project_source::validate_kind(&source.kind, &row.project_type)?;
        required(&row.title, 200, "Speech project title")?;
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
        reference(projects, &row.project_id, "speech project revision owner")?;
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
        let config =
            crate::spaces::speech::project_config::parse(project_type, &row.output_config)?;
        if let crate::spaces::speech::project_config::ProjectOutputConfig::Audiobook(config) =
            &config
        {
            for role in &config.roles {
                reference(revisions, &role.voice_revision_id, "audiobook role voice")?;
            }
            let ordered = chapter_segments
                .get(row.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            crate::spaces::speech::project_config::validate_chapter_order(
                config,
                ordered.iter().map(|segment| segment.chapter_key.as_deref()),
            )?;
            for segment in ordered {
                let role_voice = crate::spaces::speech::project_config::role_voice(
                    config,
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
        } else if let crate::spaces::speech::project_config::ProjectOutputConfig::VideoDubbing(
            config,
        ) = &config
        {
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
                    config,
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

    Ok(())
}

fn validate_project_output(
    row: &crate::spaces::package::restore::contracts::PackageSpeechProjectRevision,
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
