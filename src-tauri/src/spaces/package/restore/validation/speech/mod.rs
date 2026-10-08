mod candidates;
mod projects;
mod renders;
mod segments;
mod separation;
mod voices;
use crate::spaces::package::restore::contracts::RestorePackageRequest;
use std::collections::{HashMap, HashSet};

pub(super) fn validate(request: &RestorePackageRequest) -> Result<(), String> {
    validate_runtime_snapshots(request)?;
    let index = SpeechIndex::new(request)?;
    candidates::validate(request, &index)?;
    separation::validate(request, &index)?;
    voices::validate(request, &index)?;
    let lexicons = request
        .speech_project_revisions
        .iter()
        .map(|revision| {
            crate::spaces::speech::text::parse_lexicon(&revision.pronunciation_lexicon)
                .map(|lexicon| (revision.id.as_str(), lexicon))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    projects::validate(request, &index)?;
    segments::validate(request, &index, &lexicons)?;
    renders::validate(request, &index)
}

struct SpeechIndex<'a> {
    profiles: HashSet<&'a str>,
    revisions: HashSet<&'a str>,
    projects: HashSet<&'a str>,
    project_revisions: HashSet<&'a str>,
    segments: HashSet<&'a str>,
    revision_owners: HashMap<&'a str, &'a str>,
    voice_languages: HashMap<&'a str, &'a str>,
    voice_deliveries: HashMap<&'a str, bool>,
    project_types: HashMap<&'a str, &'a str>,
    project_revision_owners: HashMap<&'a str, &'a str>,
    separation_rows: HashMap<
        &'a str,
        &'a crate::spaces::package::restore::contracts::PackageSpeechSourceSeparation,
    >,
    run_ids: HashSet<&'a str>,
}
impl<'a> SpeechIndex<'a> {
    fn new(request: &'a RestorePackageRequest) -> Result<Self, String> {
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
        let voice_sources = request
            .voice_profiles
            .iter()
            .map(|row| (row.id.as_str(), row.source_kind.as_str()))
            .collect::<HashMap<_, _>>();
        let voice_deliveries = request
            .voice_revisions
            .iter()
            .map(|row| {
                let source = voice_sources
                    .get(row.profile_id.as_str())
                    .ok_or("Package voice profile is unavailable")?;
                Ok((
                    row.id.as_str(),
                    crate::spaces::speech::delivery_instruction::supports(
                        &row.engine_id,
                        source,
                        row.reference_text.as_deref(),
                    ),
                ))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;
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
        Ok(Self {
            profiles,
            revisions,
            projects,
            project_revisions,
            segments,
            revision_owners,
            voice_languages,
            voice_deliveries,
            project_types,
            project_revision_owners,
            separation_rows,
            run_ids,
        })
    }
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
        crate::spaces::speech::runtime_identity::read_snapshot(&run.input_snapshot)?;
    }
    Ok(())
}

fn ids<'a>(values: impl Iterator<Item = &'a str>, label: &str) -> Result<HashSet<&'a str>, String> {
    let mut unique_ids = HashSet::new();
    for value in values {
        if value.is_empty() || !unique_ids.insert(value) {
            return Err(format!("Package {label} IDs must be nonempty and unique"));
        }
    }
    Ok(unique_ids)
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
