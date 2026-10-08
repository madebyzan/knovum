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
    request: DraftVoiceCandidateRequest,
    digest: String,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<FrozenConfiguration, String> {
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let frozen = match &request.source {
        VoiceCandidateSource::Designed { .. } => None,
        VoiceCandidateSource::Reference { source_path, .. } => Some(
            freeze_reference(
                app,
                pool,
                &request,
                std::path::PathBuf::from(source_path),
                cancellation,
            )
            .await?,
        ),
    };
    let (reference_source, publication_id) = match frozen {
        Some((snapshot, publication_id)) => (Some(snapshot), Some(publication_id)),
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
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<(ReferenceSourceSnapshot, String), String> {
    crate::spaces::assets::authorize_source(app, &path)?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    validate_source_metadata(&metadata)?;
    publish_snapshot(app, pool, request, &path, metadata.len(), cancellation).await
}

async fn publish_snapshot(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    request: &DraftVoiceCandidateRequest,
    source: &std::path::Path,
    byte_length: u64,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<(ReferenceSourceSnapshot, String), String> {
    let VoiceCandidateSource::Reference {
        start_ms,
        end_ms,
        source_revision,
        ..
    } = &request.source
    else {
        return Err("Voice candidate has no reference selection".into());
    };
    let interval = (*start_ms, *end_ms);
    let directory = crate::spaces::assets::asset_directory(app)?
        .join("speech")
        .join("voice-candidate-inputs");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let target = directory.join(format!("{}.input", request.request_id));
    let part = directory.join(format!(".{}.input.part", request.request_id));
    if target.exists() || part.exists() {
        return Err("Voice candidate source snapshot path is already occupied".into());
    }
    let publication_id = super::super::asset_publication::stage_unowned(
        pool,
        &request.space_id,
        super::super::asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::VoiceCandidateInput,
            owner_id: &request.request_id,
            asset_path: &target,
            part_path: Some(&part),
            sha256: &"0".repeat(64),
        },
    )
    .await?;
    let original = source.to_owned();
    let prepared = part.clone();
    let published = target.clone();
    let reviewed_revision = source_revision.clone();
    let handle = app.clone();
    let cancel = cancellation.clone();
    let preparation: Result<ReferenceSourceSnapshot, String> = async {
        let audio = crate::blocking::run(move || -> Result<_, String> {
            let (revision, audio) = super::super::voice_assets::prepare_reference_clip(
                &handle,
                &original,
                &prepared,
                byte_length,
                &cancel,
                interval,
            )?;
            if revision != reviewed_revision {
                return Err(
                    "stale_revision: Reference changed after preparation and transcript review"
                        .into(),
                );
            }
            std::fs::rename(&prepared, &published).map_err(|error| error.to_string())?;
            Ok(audio)
        })
        .await?;
        super::super::asset_publication::seal_unowned(pool, &publication_id, &audio.sha256).await?;
        Ok(ReferenceSourceSnapshot {
            asset_path: target.to_string_lossy().into_owned(),
            byte_length: audio.byte_length,
            source_revision: format!("sha256:{}", audio.sha256),
            duration_ms: interval.1 - interval.0,
        })
    }
    .await;
    match preparation {
        Ok(snapshot) => Ok((snapshot, publication_id)),
        Err(error) => {
            super::super::asset_publication::abandon(app, pool, &publication_id)
                .await
                .map_err(|cleanup| format!("{error}; voice source cleanup failed: {cleanup}"))?;
            Err(error)
        }
    }
}

pub(super) fn validate_source_metadata(metadata: &std::fs::Metadata) -> Result<(), String> {
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Voice reference must be a regular media file".into());
    }
    if metadata.len() == 0 {
        return Err("Voice reference source is empty".into());
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
    if format!("sha256:{}", hex::encode(digest.finalize())) != revision {
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
    if format!("sha256:{}", hex::encode(digest.finalize())) != expected_revision {
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
