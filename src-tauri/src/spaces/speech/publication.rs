use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Runtime};

use super::{
    contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput},
    generation_attempt::GenerationAttempt,
};
use crate::spaces::{assets, db};

pub fn write_preview<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    audio: crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<SpeechAudioOutput, String> {
    validate_audio_payload(&audio)?;
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-previews");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(format!("{job_id}.wav"));
    atomic_write(&path, &audio.bytes)?;
    Ok(SpeechAudioOutput {
        path: path.to_string_lossy().into_owned(),
        sha256: audio.sha256,
        sample_rate: audio.sample_rate,
        sample_count: audio.sample_count,
    })
}

pub fn segment_path<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    revision_id: &str,
    segment_id: &str,
    input_digest: &str,
) -> Result<PathBuf, String> {
    for (value, label) in [
        (project_id, "project"),
        (revision_id, "revision"),
        (segment_id, "segment"),
    ] {
        uuid::Uuid::parse_str(value).map_err(|_| format!("Speech {label} identity is invalid"))?;
    }
    if input_digest.len() != 64 || !input_digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Speech input digest is invalid".to_owned());
    }
    Ok(assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id)
        .join(format!("{segment_id}-{input_digest}.wav")))
}

pub fn publish_segment(
    path: &Path,
    audio: &crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<PathBuf, String> {
    validate_audio_payload(audio)?;
    let directory = path
        .parent()
        .ok_or("Speech output path has no parent directory")?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    atomic_write(path, &audio.bytes)?;
    let actual = hash_file(path)?;
    if actual != audio.sha256 {
        let _ = fs::remove_file(path);
        return Err("invalid_output: published speech hash differs".to_owned());
    }
    Ok(path.to_owned())
}

pub fn validate_cached<R: Runtime>(
    app: &AppHandle<R>,
    output: SpeechSegmentRenderOutput,
) -> Result<SpeechSegmentRenderOutput, String> {
    let path = assets::authorize_spaces_asset(app, Path::new(&output.path))?;
    if hash_file(&path)? != output.sha256 {
        return Err("Speech render cache is corrupt".to_owned());
    }
    Ok(output)
}

pub fn discard_published_file(path: &Path) {
    let _ = fs::remove_file(path);
}

pub fn discard_preview<R: Runtime>(app: &AppHandle<R>, candidate: &str) -> Result<(), String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-previews");
    let root = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = Path::new(candidate)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if path.parent() != Some(root.as_path())
        || path.extension().and_then(|value| value.to_str()) != Some("wav")
    {
        return Err("Speech preview path is outside the managed cache".to_owned());
    }
    fs::remove_file(path).map_err(|error| error.to_string())
}

pub async fn reconcile_interrupted<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    reconcile_previews(app)?;
    let pool = db::pool(app).await?;
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id,segment_id,asset_path FROM spaces_speech_renders WHERE state='staged'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (render_id, segment_id, candidate) in rows {
        if let Some(candidate) = candidate {
            let path = Path::new(&candidate);
            if path.exists() {
                let authorized = assets::authorize_spaces_asset(app, path)?;
                fs::remove_file(authorized).map_err(|error| error.to_string())?;
            }
            let part = path.with_extension("wav.part");
            if part.exists() {
                let authorized = assets::authorize_spaces_asset(app, &part)?;
                fs::remove_file(authorized).map_err(|error| error.to_string())?;
            }
        }
        let mut tx = db::begin_write(pool).await?;
        sqlx::query(
            "UPDATE spaces_speech_renders SET state='failed',asset_path=NULL WHERE id=? AND state='staged'",
        )
        .bind(&render_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        sqlx::query(
            "UPDATE spaces_speech_segments SET state='ready',state_reason='interrupted',updated_at=? \
             WHERE id=? AND state='rendering'",
        )
        .bind(crate::clock::unix_time_millis()?)
        .bind(&segment_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        tx.commit().await.map_err(db::sql_error)?;
    }
    sqlx::query(
        "UPDATE spaces_speech_projects SET active_generation_job_id=NULL, \
         generation_cancel_requested_at=NULL WHERE active_generation_job_id IS NOT NULL",
    )
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

fn reconcile_previews<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-previews");
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let managed = matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("wav") | Some("part")
        );
        if managed && path.is_file() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

pub async fn commit_render(
    pool: &sqlx::SqlitePool,
    attempt: GenerationAttempt<'_>,
    render_id: &str,
    segment_id: &str,
    path: &Path,
    audio: &crate::ai::local_runtime::speech::SynthesizedAudio,
    is_last: bool,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    if attempt.cancellation.is_cancelled() {
        reject_cancelled(
            &mut tx,
            render_id,
            segment_id,
            attempt.job_id,
            attempt.epoch,
        )
        .await?;
        tx.commit().await.map_err(db::sql_error)?;
        let _ = fs::remove_file(path);
        return Err("cancelled".to_owned());
    }
    let current: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_speech_projects p \
         JOIN spaces_speech_project_revisions r ON r.id=p.active_revision_id \
         JOIN spaces_speech_segments s ON s.project_revision_id=r.id \
         WHERE p.id=? AND r.id=? AND s.id=? AND s.state='rendering' \
         AND p.active_generation_job_id=? AND p.generation_epoch=? \
         AND p.generation_cancel_requested_at IS NULL)",
    )
    .bind(attempt.project_id)
    .bind(attempt.revision_id)
    .bind(segment_id)
    .bind(attempt.job_id)
    .bind(attempt.epoch)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if !current {
        let rejected = sqlx::query(
            "UPDATE spaces_speech_renders SET state='rejected',asset_path=NULL \
             WHERE id=? AND state='staged' AND generation_job_id=? AND generation_epoch=?",
        )
        .bind(render_id)
        .bind(attempt.job_id)
        .bind(attempt.epoch)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        if rejected.rows_affected() == 1 {
            sqlx::query(
                "UPDATE spaces_speech_segments SET state='ready',state_reason='stale_revision',updated_at=? \
                 WHERE id=? AND state='rendering'",
            )
            .bind(crate::clock::unix_time_millis()?).bind(segment_id)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
        }
        tx.commit().await.map_err(db::sql_error)?;
        let _ = fs::remove_file(path);
        return Err("stale_revision: speech project changed before publication".to_owned());
    }
    let now = crate::clock::unix_time_millis()?;
    let published = sqlx::query(
        "UPDATE spaces_speech_renders SET state='published',asset_path=?,output_sha256=?, \
         sample_rate=?,sample_count=?,published_at=? WHERE id=? AND state='staged' \
         AND generation_job_id=? AND generation_epoch=?",
    )
    .bind(path.to_string_lossy().as_ref())
    .bind(&audio.sha256)
    .bind(i64::from(audio.sample_rate))
    .bind(
        i64::try_from(audio.sample_count)
            .map_err(|_| "Speech sample count exceeds SQLite range")?,
    )
    .bind(now)
    .bind(render_id)
    .bind(attempt.job_id)
    .bind(attempt.epoch)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        published,
        "Speech render publication lost its staged attempt",
    )?;
    let segment = sqlx::query(
        "UPDATE spaces_speech_segments SET state='rendered',state_reason=NULL,updated_at=? \
         WHERE id=? AND state='rendering'",
    )
    .bind(now)
    .bind(segment_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(segment, "Speech segment changed before publication")?;
    if attempt.cancellation.is_cancelled() {
        reject_cancelled(
            &mut tx,
            render_id,
            segment_id,
            attempt.job_id,
            attempt.epoch,
        )
        .await?;
        tx.commit().await.map_err(db::sql_error)?;
        let _ = fs::remove_file(path);
        return Err("cancelled".to_owned());
    }
    if is_last {
        super::generation_attempt::clear_generation_in_transaction(
            &mut tx,
            attempt.project_id,
            attempt.job_id,
            attempt.epoch,
        )
        .await?;
    }
    match tx.commit().await {
        Ok(()) => Ok(()),
        Err(error) => {
            let published: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM spaces_speech_renders WHERE id=? AND state='published')",
            )
            .bind(render_id)
            .fetch_one(pool)
            .await
            .map_err(db::sql_error)?;
            if published {
                Ok(())
            } else {
                Err(db::sql_error(error))
            }
        }
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err("Speech output path is already occupied".to_owned());
    }
    let part = path.with_extension("wav.part");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part)
        .map_err(|error| error.to_string())?;
    let result = (|| {
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&part, path).map_err(|error| error.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(part);
    }
    result
}

async fn reject_cancelled(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    render_id: &str,
    segment_id: &str,
    job_id: &str,
    generation_epoch: i64,
) -> Result<(), String> {
    let cancelled = sqlx::query(
        "UPDATE spaces_speech_renders SET state='cancelled',asset_path=NULL,output_sha256=NULL, \
         sample_rate=NULL,sample_count=NULL,published_at=NULL \
         WHERE id=? AND state IN ('staged','published') \
         AND generation_job_id=? AND generation_epoch=?",
    )
    .bind(render_id)
    .bind(job_id)
    .bind(generation_epoch)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if cancelled.rows_affected() == 1 {
        sqlx::query(
            "UPDATE spaces_speech_segments SET state='ready',state_reason='cancelled',updated_at=? \
             WHERE id=? AND state='rendering'",
        )
        .bind(crate::clock::unix_time_millis()?).bind(segment_id)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn validate_audio_payload(
    audio: &crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<(), String> {
    let bytes = &audio.bytes;
    if audio.sample_count == 0
        || bytes.len() < 44
        || bytes.get(0..4) != Some(b"RIFF".as_slice())
        || bytes.get(8..12) != Some(b"WAVE".as_slice())
    {
        return Err("invalid_output: synthesized WAV container is invalid".to_owned());
    }
    let declared = usize::try_from(read_u32(bytes, 4)?)
        .map_err(|_| "invalid_output: WAV size exceeds this platform")?
        .checked_add(8)
        .ok_or("invalid_output: WAV size overflow")?;
    if declared != bytes.len() {
        return Err("invalid_output: WAV container length differs".to_owned());
    }
    let mut offset = 12_usize;
    let mut format_valid = false;
    let mut data_samples = None;
    while offset.checked_add(8).is_some_and(|end| end <= bytes.len()) {
        let size = usize::try_from(read_u32(bytes, offset + 4)?)
            .map_err(|_| "invalid_output: WAV chunk exceeds this platform")?;
        let start = offset + 8;
        let end = start
            .checked_add(size)
            .ok_or("invalid_output: WAV chunk overflow")?;
        if end > bytes.len() {
            return Err("invalid_output: WAV chunk exceeds its container".to_owned());
        }
        let chunk_id = &bytes[offset..offset + 4];
        if chunk_id == b"fmt " && size >= 16 {
            format_valid = read_u16(bytes, start)? == 1
                && read_u16(bytes, start + 2)? == 1
                && read_u32(bytes, start + 4)? == audio.sample_rate
                && read_u16(bytes, start + 14)? == 16;
        } else if chunk_id == b"data" && size % 2 == 0 {
            data_samples = Some(
                u64::try_from(size / 2)
                    .map_err(|_| "invalid_output: WAV sample count exceeds protocol range")?,
            );
        }
        offset = end + (size % 2);
    }
    if !format_valid || data_samples != Some(audio.sample_count) {
        return Err("invalid_output: WAV audio metadata differs".to_owned());
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let value: [u8; 2] = bytes
        .get(offset..offset + 2)
        .ok_or("invalid_output: WAV header is truncated")?
        .try_into()
        .map_err(|_| "invalid_output: WAV header is truncated")?;
    Ok(u16::from_le_bytes(value))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value: [u8; 4] = bytes
        .get(offset..offset + 4)
        .ok_or("invalid_output: WAV header is truncated")?
        .try_into()
        .map_err(|_| "invalid_output: WAV header is truncated")?;
    Ok(u32::from_le_bytes(value))
}
