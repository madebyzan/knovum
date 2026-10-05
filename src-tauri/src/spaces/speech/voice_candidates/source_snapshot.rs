use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use std::io::{Read, Write};

use sha2::{Digest, Sha256};

use super::{
    DraftVoiceCandidateRequest, ReferenceSourceSnapshot, VoiceCandidateSource,
    VoiceCandidateStepConfiguration,
};

pub(super) struct FrozenConfiguration {
    pub configuration: VoiceCandidateStepConfiguration,
    pub request_digest: String,
    pub publication_id: Option<String>,
}

pub(super) async fn freeze(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    mut request: DraftVoiceCandidateRequest,
    digest: String,
) -> Result<FrozenConfiguration, String> {
    let frozen = match &request.source {
        VoiceCandidateSource::Designed { .. } => None,
        VoiceCandidateSource::Reference { source_path, .. } => Some(
            freeze_reference(app, pool, &request, std::path::PathBuf::from(source_path)).await?,
        ),
    };
    let (reference_source, publication_id) = match frozen {
        Some((snapshot, publication_id)) => {
            if let VoiceCandidateSource::Reference { source_path, .. } = &mut request.source {
                source_path.clone_from(&snapshot.asset_path);
            }
            (Some(snapshot), Some(publication_id))
        }
        None => (None, None),
    };
    Ok(FrozenConfiguration {
        configuration: VoiceCandidateStepConfiguration {
            request,
            reference_source,
        },
        request_digest: digest,
        publication_id,
    })
}

async fn freeze_reference(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    request: &DraftVoiceCandidateRequest,
    path: std::path::PathBuf,
) -> Result<(ReferenceSourceSnapshot, String), String> {
    crate::spaces::assets::authorize_source(app, &path)?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    validate_source_metadata(&metadata)?;
    let revision_path = path.clone();
    let source_revision =
        crate::blocking::run(move || crate::file_inspection::content_revision(&revision_path))
            .await?;
    if let VoiceCandidateSource::Reference {
        source_revision: reviewed,
        ..
    } = &request.source
    {
        if reviewed != &source_revision {
            return Err(
                "stale_revision: Reference changed after preparation and transcript review".into(),
            );
        }
    }
    let verify_path = path.clone();
    let expected_revision = source_revision.clone();
    crate::blocking::run(move || {
        crate::file_inspection::verify_content_revision(&verify_path, &expected_revision)
    })
    .await
    .map_err(|_| "stale_revision: voice reference changed while its Run was created")?;
    let after = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if after.len() != metadata.len() || validate_source_metadata(&after).is_err() {
        return Err("stale_revision: voice reference changed while its Run was created".into());
    }
    publish_snapshot(app, pool, request, &path, metadata.len(), source_revision).await
}

async fn publish_snapshot(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    request: &DraftVoiceCandidateRequest,
    source: &std::path::Path,
    byte_length: u64,
    source_revision: String,
) -> Result<(ReferenceSourceSnapshot, String), String> {
    let directory = crate::spaces::assets::asset_directory(app)?
        .join("speech")
        .join("voice-candidate-inputs");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let target = directory.join(format!("{}.input", request.request_id));
    let part = directory.join(format!(".{}.input.part", request.request_id));
    if target.exists() || part.exists() {
        return Err("Voice candidate source snapshot path is already occupied".into());
    }
    let raw_sha = source_revision
        .strip_prefix("sha256:")
        .ok_or("Voice reference source revision is invalid")?;
    let publication_id = super::super::asset_publication::stage_unowned(
        pool,
        &request.space_id,
        super::super::asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::VoiceCandidateInput,
            owner_id: &request.request_id,
            asset_path: &target,
            part_path: Some(&part),
            sha256: raw_sha,
        },
    )
    .await?;
    let source_copy = source.to_owned();
    let part_copy = part.clone();
    let target_copy = target.clone();
    let revision_copy = source_revision.clone();
    let copied = crate::blocking::run(move || {
        copy(&source_copy, &part_copy, byte_length, &revision_copy).and_then(|()| {
            std::fs::rename(&part_copy, &target_copy).map_err(|error| error.to_string())
        })
    })
    .await;
    if let Err(error) = copied {
        super::super::asset_publication::abandon(app, pool, &publication_id)
            .await
            .map_err(|cleanup| format!("{error}; voice source cleanup failed: {cleanup}"))?;
        return Err(error);
    }
    Ok((
        ReferenceSourceSnapshot {
            asset_path: target.to_string_lossy().into_owned(),
            byte_length,
            source_revision,
        },
        publication_id,
    ))
}

fn validate_source_metadata(metadata: &std::fs::Metadata) -> Result<(), String> {
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Voice reference must be a regular audio file".into());
    }
    if metadata.len() == 0 {
        return Err("Voice reference source is empty".into());
    }
    if metadata.len() > 128 * 1024 * 1024 {
        return Err("Voice reference source exceeds the 128 MiB input limit".into());
    }
    Ok(())
}

pub(super) fn copy(
    source: &std::path::Path,
    target: &std::path::Path,
    expected_byte_length: u64,
    revision: &str,
) -> Result<(), String> {
    let mut input = std::fs::File::open(source).map_err(|error| error.to_string())?;
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(target)
        .map_err(|error| error.to_string())?;
    let mut buffer = vec![0_u8; 1024 * 1024];
    let buffer_length = u64::try_from(buffer.len())
        .map_err(|_| "Voice reference copy buffer exceeds this platform")?;
    let mut digest = Sha256::new();
    let mut remaining = expected_byte_length;
    while remaining > 0 {
        let requested = usize::try_from(remaining.min(buffer_length))
            .map_err(|_| "Voice reference snapshot length exceeds this platform")?;
        let read = input
            .read(&mut buffer[..requested])
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("stale_revision: voice reference shrank while it was copied".into());
        }
        output
            .write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
        digest.update(&buffer[..read]);
        remaining = remaining
            .checked_sub(u64::try_from(read).map_err(|_| "Voice reference read length overflow")?)
            .ok_or("Voice reference snapshot length underflow")?;
    }
    let mut extra = [0_u8; 1];
    if input.read(&mut extra).map_err(|error| error.to_string())? != 0 {
        return Err("stale_revision: voice reference grew while it was copied".into());
    }
    if format!("sha256:{:x}", digest.finalize()) != revision {
        return Err("stale_revision: voice reference changed while it was copied".into());
    }
    output.sync_all().map_err(|error| error.to_string())?;
    verify_exact_revision(target, expected_byte_length, revision)
}

fn verify_exact_revision(
    path: &std::path::Path,
    expected_byte_length: u64,
    expected_revision: &str,
) -> Result<(), String> {
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    let buffer_length = u64::try_from(buffer.len())
        .map_err(|_| "Voice reference verification buffer exceeds this platform")?;
    let mut remaining = expected_byte_length;
    while remaining > 0 {
        let requested = usize::try_from(remaining.min(buffer_length))
            .map_err(|_| "Voice reference verification length exceeds this platform")?;
        let read = file
            .read(&mut buffer[..requested])
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("Voice reference snapshot is shorter than its frozen length".into());
        }
        digest.update(&buffer[..read]);
        remaining = remaining
            .checked_sub(u64::try_from(read).map_err(|_| "Voice reference read length overflow")?)
            .ok_or("Voice reference verification length underflow")?;
    }
    let mut extra = [0_u8; 1];
    if file.read(&mut extra).map_err(|error| error.to_string())? != 0 {
        return Err("Voice reference snapshot exceeds its frozen length".into());
    }
    if format!("sha256:{:x}", digest.finalize()) != expected_revision {
        return Err("Voice reference snapshot digest differs from its source".into());
    }
    Ok(())
}

pub(super) fn remove_managed_snapshot<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    run_id: &str,
    asset_path: &str,
) -> Result<(), String> {
    super::require_uuid(run_id, "Voice candidate Run identity")?;
    let expected = crate::spaces::assets::asset_directory(app)?
        .join("speech")
        .join("voice-candidate-inputs")
        .join(format!("{run_id}.input"));
    let path = std::path::Path::new(asset_path);
    if path != expected {
        return Err("Voice candidate source snapshot path is not owned by its Run".into());
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Voice candidate source snapshot is not a managed regular file".into());
    }
    let authorized = crate::spaces::assets::authorize_spaces_asset(app, path)?;
    let expected = expected.canonicalize().map_err(|error| error.to_string())?;
    if authorized != expected {
        return Err("Voice candidate source snapshot resolved outside its owned path".into());
    }
    std::fs::remove_file(authorized).map_err(|error| error.to_string())
}
