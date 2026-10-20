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
    separation::validate(request, &index)?;
    voices::validate(request, &index)?;
    let lexicon_by_project_revision_id = request
        .speech_project_revisions
        .iter()
        .map(|revision| {
            crate::spaces::speech::text::parse_lexicon(&revision.pronunciation_lexicon)
                .map(|lexicon| (revision.id.as_str(), lexicon))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    projects::validate(request, &index)?;
    segments::validate(request, &index, &lexicon_by_project_revision_id)?;
    renders::validate(request, &index)
}

struct SpeechIndex<'a> {
    profiles: HashSet<&'a str>,
    revisions: HashSet<&'a str>,
    projects: HashSet<&'a str>,
    project_revisions: HashSet<&'a str>,
    segments: HashSet<&'a str>,
    profile_id_by_revision_id: HashMap<&'a str, &'a str>,
    language_by_revision_id: HashMap<&'a str, &'a str>,
    delivery_by_revision_id: HashMap<&'a str, bool>,
    project_type_by_id: HashMap<&'a str, &'a str>,
    project_id_by_revision_id: HashMap<&'a str, &'a str>,
    separation_by_id: HashMap<
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

        let profile_id_by_revision_id = request
            .voice_revisions
            .iter()
            .map(|row| (row.id.as_str(), row.profile_id.as_str()))
            .collect::<HashMap<_, _>>();
        let language_by_revision_id = request
            .voice_revisions
            .iter()
            .map(|row| (row.id.as_str(), row.language.as_str()))
            .collect::<HashMap<_, _>>();
        let source_by_profile_id = request
            .voice_profiles
            .iter()
            .map(|row| (row.id.as_str(), row.source_kind.as_str()))
            .collect::<HashMap<_, _>>();
        let delivery_by_revision_id = request
            .voice_revisions
            .iter()
            .map(|row| {
                let source = source_by_profile_id
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
        let project_type_by_id = request
            .speech_projects
            .iter()
            .map(|row| (row.id.as_str(), row.project_type.as_str()))
            .collect::<HashMap<_, _>>();
        let project_id_by_revision_id = request
            .speech_project_revisions
            .iter()
            .map(|row| (row.id.as_str(), row.project_id.as_str()))
            .collect::<HashMap<_, _>>();
        let separation_by_id = request
            .speech_source_separations
            .iter()
            .map(|row| (row.id.as_str(), row))
            .collect::<HashMap<_, _>>();
        let run_ids = request
            .runs
            .iter()
            .map(|row| row.id.as_str())
            .collect::<HashSet<_>>();
        Ok(Self {
            profiles,
            revisions,
            projects,
            project_revisions,
            segments,
            profile_id_by_revision_id,
            language_by_revision_id,
            delivery_by_revision_id,
            project_type_by_id,
            project_id_by_revision_id,
            separation_by_id,
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
