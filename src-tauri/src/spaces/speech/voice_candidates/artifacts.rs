use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use sqlx::SqlitePool;

use super::{repository, VoiceCandidateSource, VoiceCandidateStepConfiguration};
use crate::spaces::{assets, runs::AttemptClaim};

pub(super) async fn verify_source_snapshot(
    app: &tauri::AppHandle,
    configuration: &VoiceCandidateStepConfiguration,
) -> Result<(), String> {
    match (
        &configuration.request.source,
        &configuration.reference_source,
    ) {
        (VoiceCandidateSource::Designed { .. }, None) => Ok(()),
        (VoiceCandidateSource::Reference { source_path, .. }, Some(snapshot)) => {
            let path = PathBuf::from(source_path);
            let path = assets::authorize_spaces_asset(app, &path)?;
            let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() != snapshot.byte_length
            {
                return Err("stale_revision: voice reference changed before execution".into());
            }
            let expected = snapshot.source_revision.clone();
            crate::blocking::run(move || {
                crate::file_inspection::verify_content_revision(&path, &expected)
            })
            .await
            .map_err(|_| "stale_revision: voice reference changed before execution".into())
        }
        _ => Err("Voice candidate reference snapshot does not match its source".into()),
    }
}

pub(super) fn cleanup_source_snapshot(
    app: &tauri::AppHandle,
    configuration: &VoiceCandidateStepConfiguration,
) -> Result<(), String> {
    let Some(source) = &configuration.reference_source else {
        return Ok(());
    };
    super::source_snapshot::remove_managed_snapshot(
        app,
        &configuration.request.request_id,
        &source.asset_path,
    )
}

pub(super) fn validate_persisted(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    record: &repository::CandidateRecord,
) -> Result<(), String> {
    if record.candidate.run_id != claim.run_id || record.candidate.space_id != claim.home_space_id {
        return Err("stale_revision: persisted voice candidate differs from its Run".into());
    }
    for (path, expected) in [
        (&record.candidate.audio.path, &record.candidate.audio.sha256),
        (&record.reference_asset_path, &record.reference_sha256),
    ] {
        let path = assets::authorize_spaces_asset(app, Path::new(path))?;
        if super::super::publication::hash_file(&path)? != *expected {
            return Err("stale_revision: persisted voice candidate asset changed".into());
        }
    }
    Ok(())
}

pub(super) async fn stage(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    asset: &Path,
    part: &Path,
    sha256: &str,
) -> Result<(), String> {
    super::super::asset_publication::stage_for_attempt(
        pool,
        claim,
        super::super::asset_publication::PendingAsset {
            owner_kind: "voice_candidate",
            owner_id: &claim.run_id,
            asset_path: asset,
            part_path: Some(part),
            sha256,
        },
    )
    .await
    .map(|_| ())
}

pub(super) fn write_new(
    path: &Path,
    bytes: &[u8],
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    for chunk in bytes.chunks(1024 * 1024) {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        output.write_all(chunk).map_err(|error| error.to_string())?;
    }
    output.sync_all().map_err(|error| error.to_string())
}

pub(super) struct CandidatePaths {
    pub preview: PathBuf,
    pub preview_part: PathBuf,
    pub reference: PathBuf,
    pub reference_part: PathBuf,
}

impl CandidatePaths {
    pub fn new(app: &tauri::AppHandle, claim: &AttemptClaim) -> Result<Self, String> {
        let directory = assets::asset_directory(app)?
            .join("speech")
            .join("voice-candidates");
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        Ok(Self {
            preview: directory.join(format!("{}-preview.wav", claim.run_id)),
            preview_part: directory.join(format!(".{}-preview.wav.part", claim.attempt_id)),
            reference: directory.join(format!("{}-reference.wav", claim.run_id)),
            reference_part: directory.join(format!(".{}-reference.wav.part", claim.attempt_id)),
        })
    }

    pub fn require_available(&self) -> Result<(), String> {
        if [
            &self.preview,
            &self.preview_part,
            &self.reference,
            &self.reference_part,
        ]
        .into_iter()
        .any(|path| path.exists())
        {
            return Err("Voice candidate output path is already occupied".into());
        }
        Ok(())
    }

    pub fn cleanup_working(&self) {
        for path in [&self.preview_part, &self.reference_part] {
            let _ = fs::remove_file(path);
        }
    }

    pub fn cleanup_published(&self) {
        for path in [&self.preview, &self.reference] {
            let _ = fs::remove_file(path);
        }
    }
}
