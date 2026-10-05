use super::{
    output_spec::{digest_render_order, output_profile, ProjectOutputProfile},
    ProjectOutputRequest,
};
use crate::spaces::{
    assets,
    runs::AttemptClaim,
    speech::{contracts::SpeechSegmentRenderOutput, publication::hash_file},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

pub(super) struct ProjectInputs {
    pub profile: ProjectOutputProfile,
    pub digest: String,
    pub directory: PathBuf,
}

pub(super) fn prepare<R: Runtime>(
    app: &AppHandle<R>,
    claim: &AttemptClaim,
    request: &ProjectOutputRequest<'_>,
) -> Result<ProjectInputs, String> {
    for (id, label) in [
        (request.project_id, "project"),
        (request.revision_id, "revision"),
        (&claim.attempt_id, "assembly attempt"),
    ] {
        uuid::Uuid::parse_str(id).map_err(|_| format!("Speech {label} identity is invalid"))?;
    }
    if request.renders.is_empty() || request.chapter_keys.len() != request.renders.len() {
        return Err("Speech project has no complete ordered render set".into());
    }
    let profile = output_profile(request.project_type, request.output_config, request.renders)?;
    if let Some(config) = &profile.audiobook {
        crate::spaces::speech::project_config::validate_chapter_order(
            config,
            request.chapter_keys.iter().map(Option::as_deref),
        )?;
    }
    verify_cover(app, &profile)?;
    let digest = digest_render_order(request.renders, request.output_config);
    let directory = assets::asset_directory(app)?
        .join("speech")
        .join(request.project_id)
        .join(request.revision_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(ProjectInputs {
        profile,
        digest,
        directory,
    })
}

pub(super) fn authorize_sources<R: Runtime>(
    app: &AppHandle<R>,
    renders: &[SpeechSegmentRenderOutput],
    cancellation: &CancellationToken,
) -> Result<Vec<PathBuf>, String> {
    let paths = renders
        .iter()
        .map(|render| {
            if cancellation.is_cancelled() {
                return Err("cancelled".into());
            }
            assets::authorize_spaces_asset(app, Path::new(&render.path))
        })
        .collect::<Result<Vec<_>, String>>()?;
    verify_sources(&paths, renders, cancellation)?;
    Ok(paths)
}

pub(super) fn verify_sources(
    paths: &[PathBuf],
    renders: &[SpeechSegmentRenderOutput],
    cancellation: &CancellationToken,
) -> Result<(), String> {
    for (path, render) in paths.iter().zip(renders) {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        if hash_file(path)? != render.sha256 {
            return Err("Speech segment digest differs from its render metadata".into());
        }
    }
    Ok(())
}

pub(super) fn verify_cover<R: Runtime>(
    app: &AppHandle<R>,
    profile: &ProjectOutputProfile,
) -> Result<(), String> {
    if let Some(cover) = profile
        .audiobook
        .as_ref()
        .and_then(|config| config.metadata.cover.as_ref())
    {
        let path = assets::authorize_spaces_asset(app, Path::new(&cover.asset_path))?;
        crate::file_inspection::verify_content_revision(&path, &cover.source_revision)?;
    }
    Ok(())
}
