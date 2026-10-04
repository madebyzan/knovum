mod audiobook;
mod encoding;

use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use self::encoding::{
    encode, normalize_sources, wav_sample_count, write_chapter_metadata, write_concat_file,
    EncodeRequest, LoudnessMeasurement,
};
use super::{
    contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput},
    project_config::AudiobookOutputConfig,
    publication::hash_file,
};
use crate::{
    local_resources::tools,
    spaces::{assets, db, runs::AttemptClaim},
};
pub(super) use audiobook::{
    cleanup_audiobook_checkpoints, measure_audiobook, publish_audiobook,
    publish_chapter_checkpoint, AudiobookInput, ChapterCheckpoint, ChapterPublicationRequest,
    LoudnessCheckpoint,
};

pub(in crate::spaces::speech) struct ProjectOutputRequest<'a> {
    pub project_id: &'a str,
    pub revision_id: &'a str,
    pub project_type: &'a str,
    pub output_config: &'a serde_json::Value,
    pub chapter_keys: &'a [Option<String>],
    pub renders: &'a [SpeechSegmentRenderOutput],
}

#[derive(sqlx::FromRow)]
struct StoredProjectOutput {
    output_asset_path: Option<String>,
    output_sha256: Option<String>,
    output_sample_rate: Option<i64>,
    output_sample_count: Option<i64>,
}

pub(in crate::spaces::speech) async fn publish_project<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    request: &ProjectOutputRequest<'_>,
    cancellation: &CancellationToken,
) -> Result<SpeechAudioOutput, String> {
    publish_project_with_measurement(app, pool, claim, request, None, cancellation).await
}

async fn publish_project_with_measurement<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    request: &ProjectOutputRequest<'_>,
    measurement: Option<LoudnessMeasurement>,
    cancellation: &CancellationToken,
) -> Result<SpeechAudioOutput, String> {
    let project_id = request.project_id;
    let revision_id = request.revision_id;
    let project_type = request.project_type;
    let output_config = request.output_config;
    let chapter_keys = request.chapter_keys;
    let renders = request.renders;
    uuid::Uuid::parse_str(project_id).map_err(|_| "Speech project identity is invalid")?;
    uuid::Uuid::parse_str(revision_id).map_err(|_| "Speech revision identity is invalid")?;
    uuid::Uuid::parse_str(&claim.attempt_id)
        .map_err(|_| "Speech assembly attempt identity is invalid")?;
    if renders.is_empty() || chapter_keys.len() != renders.len() {
        return Err("Speech project has no complete ordered render set".into());
    }
    let (format, sample_rate, audiobook) = output_profile(project_type, output_config, renders)?;
    if let Some(config) = audiobook.as_ref() {
        super::project_config::validate_chapter_order(
            config,
            chapter_keys.iter().map(Option::as_deref),
        )?;
    }
    let digest = digest_render_order(renders, output_config);
    let directory = assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    if let Some(output) = reusable_committed_output(
        app,
        pool,
        project_id,
        revision_id,
        &digest,
        &format,
        sample_rate,
    )
    .await?
    {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        return Ok(output);
    }
    let output_path = directory.join(format!("full-{digest}-{}.{format}", claim.attempt_id));
    let sources = renders
        .iter()
        .map(|render| {
            if cancellation.is_cancelled() {
                return Err("cancelled".into());
            }
            let path = assets::authorize_spaces_asset(app, Path::new(&render.path))?;
            if hash_file(&path)? != render.sha256 {
                return Err("Speech segment digest differs from its render metadata".into());
            }
            Ok(path)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?
        .ok_or("The bundled FFmpeg tool is unavailable")?;
    let work_directory = directory.join(format!("assembly-{}", claim.attempt_id));
    if work_directory.exists() {
        fs::remove_dir_all(&work_directory).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&work_directory).map_err(|error| error.to_string())?;
    let part = part_path(&output_path);
    let mut publication_id = None;
    let assembled: Result<SpeechAudioOutput, String> = async {
        if part.exists() {
            fs::remove_file(&part).map_err(|error| error.to_string())?;
        }
        let normalize_ffmpeg = ffmpeg.clone();
        let input_sources = sources.clone();
        let normalize_renders = renders.to_vec();
        let normalize_directory = work_directory.clone();
        let normalize_cancellation = cancellation.clone();
        let normalized = crate::blocking::run(move || {
            normalize_sources(
                &normalize_ffmpeg,
                &input_sources,
                &normalize_renders,
                sample_rate,
                &normalize_directory,
                &normalize_cancellation,
            )
        })
        .await?;
        let sample_count = normalized.iter().try_fold(0_u64, |total, source| {
            total
                .checked_add(source.sample_count)
                .ok_or_else(|| "Speech project sample count overflow".to_owned())
        })?;
        let concat_path = write_concat_file(&work_directory, &normalized)?;
        let metadata_path = if let Some(config) = audiobook.as_ref().filter(|_| format == "m4b") {
            Some(write_chapter_metadata(
                &work_directory,
                config,
                chapter_keys,
                &normalized,
                sample_rate,
            )?)
        } else {
            None
        };
        let encode_ffmpeg = ffmpeg;
        let encode_concat = concat_path;
        let encode_output = part.clone();
        let encode_format = format;
        let encode_audiobook = audiobook;
        let encode_metadata = metadata_path;
        let encode_cancellation = cancellation.clone();
        crate::blocking::run(move || {
            encode(
                EncodeRequest {
                    ffmpeg: &encode_ffmpeg,
                    concat_path: &encode_concat,
                    output: &encode_output,
                    format: &encode_format,
                    sample_rate,
                    audiobook: encode_audiobook.as_ref(),
                    measurement: measurement.as_ref(),
                    metadata: encode_metadata.as_deref(),
                },
                &encode_cancellation,
            )
        })
        .await?;
        for (source, render) in sources.iter().zip(renders) {
            if cancellation.is_cancelled() {
                return Err("cancelled".into());
            }
            if hash_file(source)? != render.sha256 {
                return Err("Speech segment changed while the project was being encoded".into());
            }
        }
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        if cfg!(windows) && output_path.exists() {
            fs::remove_file(&output_path).map_err(|error| error.to_string())?;
        }
        let sha256 = hash_file(&part)?;
        let sample_count =
            if output_path.extension().and_then(|value| value.to_str()) == Some("wav") {
                wav_sample_count(&part, sample_rate)?
            } else {
                sample_count
            };
        publication_id = Some(
            super::asset_publication::stage_for_attempt(
                pool,
                claim,
                super::asset_publication::PendingAsset {
                    owner_kind: "project_output",
                    owner_id: revision_id,
                    asset_path: &output_path,
                    part_path: Some(&part),
                    sha256: &sha256,
                },
            )
            .await?,
        );
        fs::rename(&part, &output_path).map_err(|error| error.to_string())?;
        if hash_file(&output_path)? != sha256 {
            return Err("Published speech project hash changed during atomic publication".into());
        }
        Ok(SpeechAudioOutput {
            path: output_path.to_string_lossy().into_owned(),
            sha256,
            sample_rate,
            sample_count,
        })
    }
    .await;
    let assembled = finish_assembly(assembled, &part, &work_directory);
    match assembled {
        Ok(output) => Ok(output),
        Err(mut error) => {
            if let Some(publication_id) = publication_id {
                if let Err(cleanup_error) =
                    super::asset_publication::abandon(app, pool, &publication_id).await
                {
                    error.push_str("; failed to abandon project publication: ");
                    error.push_str(&cleanup_error);
                }
            } else if let Err(cleanup_error) = fs::remove_file(&output_path) {
                if cleanup_error.kind() != std::io::ErrorKind::NotFound {
                    error.push_str(&format!(
                        "; failed to remove {}: {cleanup_error}",
                        output_path.display()
                    ));
                }
            }
            Err(error)
        }
    }
}

async fn reusable_committed_output<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    project_id: &str,
    revision_id: &str,
    digest: &str,
    format: &str,
    sample_rate: u32,
) -> Result<Option<SpeechAudioOutput>, String> {
    let stored = sqlx::query_as::<_, StoredProjectOutput>(
        "SELECT output_asset_path,output_sha256,output_sample_rate,output_sample_count \
         FROM spaces_speech_project_revisions WHERE id=? AND project_id=?",
    )
    .bind(revision_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some(StoredProjectOutput {
        output_asset_path: Some(path),
        output_sha256: Some(sha256),
        output_sample_rate: Some(rate),
        output_sample_count: Some(samples),
    }) = stored
    else {
        return Ok(None);
    };
    if rate != i64::from(sample_rate) || samples < 0 {
        return Ok(None);
    }
    let source = Path::new(&path);
    let Some(file_name) = source.file_name().and_then(|value| value.to_str()) else {
        return Ok(None);
    };
    let prefix = format!("full-{digest}-");
    let suffix = format!(".{format}");
    let Some(attempt) = file_name
        .strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix(&suffix))
    else {
        return Ok(None);
    };
    if uuid::Uuid::parse_str(attempt).is_err() {
        return Ok(None);
    }
    let canonical = match assets::authorize_spaces_asset(app, source) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };
    let Ok(actual_hash) = hash_file(&canonical) else {
        return Ok(None);
    };
    if actual_hash != sha256 {
        return Ok(None);
    }
    let sample_count =
        u64::try_from(samples).map_err(|_| "Stored speech sample count is invalid")?;
    if format == "wav" {
        let Ok(actual_samples) = wav_sample_count(&canonical, sample_rate) else {
            return Ok(None);
        };
        if actual_samples != sample_count {
            return Ok(None);
        }
    }
    Ok(Some(SpeechAudioOutput {
        path: canonical.to_string_lossy().into_owned(),
        sha256,
        sample_rate,
        sample_count,
    }))
}

fn output_profile(
    project_type: &str,
    output_config: &serde_json::Value,
    renders: &[SpeechSegmentRenderOutput],
) -> Result<(String, u32, Option<AudiobookOutputConfig>), String> {
    match project_type {
        "short_dubbing" => {
            super::project_config::validate(project_type, output_config)?;
            let first = renders
                .first()
                .ok_or("Speech project has no rendered segments")?
                .sample_rate;
            let rate = if renders.iter().all(|render| render.sample_rate == first) {
                first
            } else {
                48_000
            };
            Ok(("wav".into(), rate, None))
        }
        "audiobook" => {
            let config = super::project_config::parse_audiobook(output_config)?;
            Ok((config.format.clone(), config.sample_rate, Some(config)))
        }
        _ => Err("This speech project type has no complete production workflow".into()),
    }
}

pub async fn commit_project_output(
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    project_id: &str,
    revision_id: &str,
    output: &SpeechAudioOutput,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let was_referenced: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_speech_project_revisions          WHERE id=? AND project_id=? AND output_asset_path=? AND output_sha256=?)",
    ).bind(revision_id).bind(project_id).bind(&output.path).bind(&output.sha256)
    .fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    if claim.cancellation.is_cancelled() {
        if !was_referenced {
            let _ = fs::remove_file(&output.path);
        }
        return Err("cancelled".into());
    }
    let now = crate::clock::unix_time_millis()?;
    let updated = sqlx::query(
        "UPDATE spaces_speech_project_revisions SET output_asset_path=?,output_sha256=?,          output_sample_rate=?,output_sample_count=?,output_published_at=? WHERE id=? AND project_id=?          AND EXISTS(SELECT 1 FROM spaces_speech_projects p WHERE p.id=? AND p.active_revision_id=?)          AND EXISTS(SELECT 1 FROM spaces_run_attempts a JOIN spaces_runs run ON run.id=a.run_id            WHERE a.id=? AND a.run_id=? AND a.step_instance_id=? AND a.state='executing'              AND a.execution_epoch=? AND a.lease_until>? AND run.execution_epoch=a.execution_epoch              AND run.cancel_requested_at IS NULL AND run.completed_at IS NULL)          AND NOT EXISTS(SELECT 1 FROM spaces_speech_segments WHERE project_revision_id=? AND state<>'rendered')",
    )
    .bind(&output.path).bind(&output.sha256).bind(i64::from(output.sample_rate))
    .bind(i64::try_from(output.sample_count).map_err(|_| "Speech output sample count exceeds SQLite range")?)
    .bind(now).bind(revision_id).bind(project_id).bind(project_id).bind(revision_id)
    .bind(&claim.attempt_id).bind(&claim.run_id).bind(&claim.step_instance_id)
    .bind(claim.execution_epoch).bind(now).bind(revision_id)
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
    if !was_referenced {
        super::asset_publication::remove_attempt_in_transaction(
            &mut tx,
            claim,
            Path::new(&output.path),
            &output.sha256,
        )
        .await?;
    }
    if let Err(error) = tx.commit().await {
        let committed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM spaces_speech_project_revisions              WHERE id=? AND project_id=? AND output_asset_path=? AND output_sha256=?)",
        ).bind(revision_id).bind(project_id).bind(&output.path).bind(&output.sha256)
        .fetch_one(pool).await.map_err(db::sql_error)?;
        if !committed {
            if !was_referenced {
                let _ = fs::remove_file(&output.path);
            }
            return Err(db::sql_error(error));
        }
    }
    Ok(())
}

fn digest_render_order(
    renders: &[SpeechSegmentRenderOutput],
    output_config: &serde_json::Value,
) -> String {
    let mut digest = Sha256::new();
    for render in renders {
        digest.update(render.segment_id.as_bytes());
        digest.update([0]);
        digest.update(render.sha256.as_bytes());
        digest.update([0]);
    }
    if let Ok(encoded) = serde_json::to_vec(output_config) {
        digest.update(encoded);
    }
    format!("{:x}", digest.finalize())
}

fn part_path(path: &Path) -> PathBuf {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("audio");
    path.with_extension(format!("{extension}.part"))
}

fn finish_assembly(
    result: Result<SpeechAudioOutput, String>,
    part: &Path,
    work_directory: &Path,
) -> Result<SpeechAudioOutput, String> {
    let mut cleanup_errors = Vec::new();
    if let Err(error) = fs::remove_file(part) {
        if error.kind() != std::io::ErrorKind::NotFound {
            cleanup_errors.push(format!("failed to remove {}: {error}", part.display()));
        }
    }
    if let Err(error) = fs::remove_dir_all(work_directory) {
        if error.kind() != std::io::ErrorKind::NotFound {
            cleanup_errors.push(format!(
                "failed to remove {}: {error}",
                work_directory.display()
            ));
        }
    }
    match (result, cleanup_errors.is_empty()) {
        (Ok(output), true) => Ok(output),
        (Ok(_), false) => Err(cleanup_errors.join("; ")),
        (Err(error), true) => Err(error),
        (Err(mut error), false) => {
            error.push_str("; ");
            error.push_str(&cleanup_errors.join("; "));
            Err(error)
        }
    }
}
