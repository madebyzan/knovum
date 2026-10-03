use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput},
    publication::hash_file,
};
use crate::spaces::{assets, db};

pub async fn publish_project<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    project_id: &str,
    revision_id: &str,
    renders: &[SpeechSegmentRenderOutput],
) -> Result<SpeechAudioOutput, String> {
    uuid::Uuid::parse_str(project_id).map_err(|_| "Speech project identity is invalid")?;
    uuid::Uuid::parse_str(revision_id).map_err(|_| "Speech revision identity is invalid")?;
    let first = renders
        .first()
        .ok_or("Speech project has no rendered segments")?;
    if renders
        .iter()
        .any(|render| render.sample_rate != first.sample_rate)
    {
        return Err("Speech project segments use different sample rates".to_owned());
    }
    let sample_count = renders.iter().try_fold(0_u64, |total, render| {
        total
            .checked_add(render.sample_count)
            .ok_or("Speech project sample count overflow")
    })?;
    let data_bytes = sample_count
        .checked_mul(2)
        .ok_or("Speech project audio size overflow")?;
    let data_size =
        u32::try_from(data_bytes).map_err(|_| "Speech project exceeds WAV size limits")?;
    let digest = digest_render_order(renders);
    let path = assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id)
        .join(format!("full-{digest}.wav"));
    let parent = path
        .parent()
        .ok_or("Speech project output has no parent directory")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    if path.exists() {
        let existing: Option<(String, i64, i64)> = sqlx::query_as(
            "SELECT output_sha256,output_sample_rate,output_sample_count \
             FROM spaces_speech_project_revisions WHERE id=? AND project_id=? AND output_asset_path=?",
        )
        .bind(revision_id).bind(project_id).bind(path.to_string_lossy().as_ref())
        .fetch_optional(pool).await.map_err(db::sql_error)?;
        let actual_hash = hash_file(&path)?;
        if existing
            .as_ref()
            .is_some_and(|(sha256, sample_rate, stored_samples)| {
                sha256 == &actual_hash
                    && *sample_rate == i64::from(first.sample_rate)
                    && u64::try_from(*stored_samples).ok() == Some(sample_count)
            })
        {
            return Ok(SpeechAudioOutput {
                path: path.to_string_lossy().into_owned(),
                sha256: actual_hash,
                sample_rate: first.sample_rate,
                sample_count,
            });
        }
    }
    let part = path.with_extension("wav.part");
    if part.exists() {
        fs::remove_file(&part).map_err(|error| error.to_string())?;
    }
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&part)
        .map_err(|error| error.to_string())?;
    let result = (|| {
        write_wav_header(&mut output, first.sample_rate, data_size)?;
        for render in renders {
            let source = assets::authorize_spaces_asset(app, Path::new(&render.path))?;
            if hash_file(&source)? != render.sha256 {
                return Err("Speech segment digest differs from its render metadata".to_owned());
            }
            let (offset, length) = wav_data_range(&source)?;
            if length
                != render
                    .sample_count
                    .checked_mul(2)
                    .ok_or("Speech segment size overflow")?
            {
                return Err("Speech segment payload length differs from its metadata".to_owned());
            }
            let mut input = fs::File::open(source).map_err(|error| error.to_string())?;
            input
                .seek(SeekFrom::Start(offset))
                .map_err(|error| error.to_string())?;
            std::io::copy(&mut input.take(length), &mut output)
                .map_err(|error| error.to_string())?;
        }
        output.sync_all().map_err(|error| error.to_string())?;
        drop(output);
        if cfg!(windows) && path.exists() {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
        }
        fs::rename(&part, &path).map_err(|error| error.to_string())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&part);
        return Err(error);
    }
    let sha256 = hash_file(&path)?;
    Ok(SpeechAudioOutput {
        path: path.to_string_lossy().into_owned(),
        sha256,
        sample_rate: first.sample_rate,
        sample_count,
    })
}

pub async fn commit_project_output(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    revision_id: &str,
    job_id: &str,
    generation_epoch: i64,
    output: &SpeechAudioOutput,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let was_referenced: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_speech_project_revisions \
         WHERE id=? AND project_id=? AND output_asset_path=? AND output_sha256=?)",
    )
    .bind(revision_id)
    .bind(project_id)
    .bind(&output.path)
    .bind(&output.sha256)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if cancellation.is_cancelled() {
        if !was_referenced {
            let _ = fs::remove_file(&output.path);
        }
        return Err("cancelled".to_owned());
    }
    let updated = sqlx::query(
        "UPDATE spaces_speech_project_revisions SET output_asset_path=?,output_sha256=?, \
         output_sample_rate=?,output_sample_count=?,output_published_at=? WHERE id=? AND project_id=? \
         AND EXISTS(SELECT 1 FROM spaces_speech_projects WHERE id=? AND active_revision_id=? \
           AND active_generation_job_id=? AND generation_epoch=? \
           AND generation_cancel_requested_at IS NULL) \
         AND NOT EXISTS(SELECT 1 FROM spaces_speech_segments \
           WHERE project_revision_id=? AND state<>'rendered')",
    )
    .bind(&output.path).bind(&output.sha256).bind(i64::from(output.sample_rate))
    .bind(i64::try_from(output.sample_count).map_err(|_| "Speech output sample count exceeds SQLite range")?)
    .bind(crate::clock::unix_time_millis()?).bind(revision_id).bind(project_id)
    .bind(project_id).bind(revision_id).bind(job_id).bind(generation_epoch).bind(revision_id)
    .execute(&mut *tx).await.map_err(db::sql_error)?;
    if let Err(error) = db::require_one(
        updated,
        "Speech project lost output publication eligibility",
    ) {
        if !was_referenced {
            let _ = fs::remove_file(&output.path);
        }
        return Err(error);
    }
    if cancellation.is_cancelled() {
        if !was_referenced {
            let _ = fs::remove_file(&output.path);
        }
        return Err("cancelled".to_owned());
    }
    super::generation_attempt::clear_generation_in_transaction(
        &mut tx,
        project_id,
        job_id,
        generation_epoch,
    )
    .await?;
    if let Err(error) = tx.commit().await {
        let committed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM spaces_speech_project_revisions r \
             JOIN spaces_speech_projects p ON p.id=r.project_id \
             WHERE r.id=? AND r.project_id=? AND r.output_asset_path=? AND r.output_sha256=? \
             AND p.active_generation_job_id IS NULL)",
        )
        .bind(revision_id)
        .bind(project_id)
        .bind(&output.path)
        .bind(&output.sha256)
        .fetch_one(pool)
        .await
        .map_err(db::sql_error)?;
        if !committed {
            if !was_referenced {
                let _ = fs::remove_file(&output.path);
            }
            return Err(db::sql_error(error));
        }
    }
    Ok(())
}

fn digest_render_order(renders: &[SpeechSegmentRenderOutput]) -> String {
    let mut digest = Sha256::new();
    for render in renders {
        digest.update(render.segment_id.as_bytes());
        digest.update([0]);
        digest.update(render.sha256.as_bytes());
        digest.update([0]);
    }
    format!("{:x}", digest.finalize())
}

fn write_wav_header(output: &mut fs::File, sample_rate: u32, data_size: u32) -> Result<(), String> {
    let riff_size = data_size
        .checked_add(36)
        .ok_or("Speech WAV size overflow")?;
    let byte_rate = sample_rate
        .checked_mul(2)
        .ok_or("Speech WAV byte rate overflow")?;
    output
        .write_all(b"RIFF")
        .map_err(|error| error.to_string())?;
    output
        .write_all(&riff_size.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(b"WAVEfmt ")
        .map_err(|error| error.to_string())?;
    output
        .write_all(&16_u32.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(&1_u16.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(&1_u16.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(&sample_rate.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(&byte_rate.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(&2_u16.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(&16_u16.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(b"data")
        .map_err(|error| error.to_string())?;
    output
        .write_all(&data_size.to_le_bytes())
        .map_err(|error| error.to_string())
}

fn wav_data_range(path: &Path) -> Result<(u64, u64), String> {
    let mut input = fs::File::open(path).map_err(|error| error.to_string())?;
    let length = input.metadata().map_err(|error| error.to_string())?.len();
    let mut header = [0_u8; 12];
    input
        .read_exact(&mut header)
        .map_err(|error| error.to_string())?;
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err("Speech segment WAV container is invalid".to_owned());
    }
    let mut offset = 12_u64;
    while offset.checked_add(8).is_some_and(|end| end <= length) {
        input
            .seek(SeekFrom::Start(offset))
            .map_err(|error| error.to_string())?;
        let mut chunk = [0_u8; 8];
        input
            .read_exact(&mut chunk)
            .map_err(|error| error.to_string())?;
        let size = u64::from(u32::from_le_bytes(
            chunk[4..8].try_into().map_err(|_| "Invalid WAV chunk")?,
        ));
        let start = offset + 8;
        if &chunk[0..4] == b"data" && start.checked_add(size).is_some_and(|end| end <= length) {
            return Ok((start, size));
        }
        offset = start
            .checked_add(size)
            .and_then(|end| end.checked_add(size % 2))
            .ok_or("Speech WAV chunk offset overflow")?;
    }
    Err("Speech segment WAV data chunk is missing".to_owned())
}
