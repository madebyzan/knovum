use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::encoding::{
    measure_loudness, wav_sample_count, write_concat_file, LoudnessMeasurement, NormalizedSource,
};
use super::{chapter_digest, cleanup, AudiobookInput, LoudnessCheckpoint};
use crate::spaces::speech::{project_config, publication::hash_file};
use crate::{
    local_resources::tools,
    spaces::{assets, db, runs::AttemptClaim},
};

pub(in crate::spaces::speech) async fn measure_audiobook<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    input: &AudiobookInput<'_>,
    cancellation: &CancellationToken,
) -> Result<LoudnessCheckpoint, String> {
    let project_id = input.project_id;
    let revision_id = input.revision_id;
    let output_config = input.output_config;
    let chapters = input.chapters;
    let config = project_config::parse_audiobook(output_config)?;
    let digest = chapter_digest(chapters, output_config);
    let revision_directory = assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id);
    fs::create_dir_all(&revision_directory).map_err(|error| error.to_string())?;
    cleanup::loudness_workspaces(&revision_directory)?;
    let normalized = chapters
        .iter()
        .map(|chapter| {
            let path = assets::authorize_spaces_asset(app, Path::new(&chapter.path))?;
            if hash_file(&path)? != chapter.sha256
                || wav_sample_count(
                    &path,
                    chapter.sample_rate,
                    super::super::super::audio_format::WavPrecision::Float32,
                )? != chapter.sample_count
                || chapter.sample_rate != config.sample_rate
            {
                return Err("Audiobook chapter checkpoint is corrupt".into());
            }
            Ok(NormalizedSource {
                path,
                sample_count: chapter.sample_count,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if let Some(encoded) = sqlx::query_scalar::<_, String>(
        "SELECT measurement FROM spaces_speech_loudness_checkpoints \
         WHERE project_revision_id=? AND input_digest=?",
    )
    .bind(revision_id)
    .bind(&digest)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    {
        if let Ok(measurement) = serde_json::from_str::<LoudnessMeasurement>(&encoded) {
            if let Ok(measurement) = measurement.validate() {
                return Ok(LoudnessCheckpoint {
                    input_digest: digest,
                    measurement,
                });
            }
        }
        sqlx::query(
            "DELETE FROM spaces_speech_loudness_checkpoints \
             WHERE project_revision_id=? AND input_digest=?",
        )
        .bind(revision_id)
        .bind(&digest)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    }
    let directory = revision_directory.join(format!("loudness-{}", claim.attempt_id));
    if directory.exists() {
        fs::remove_dir_all(&directory).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let normalization: Result<LoudnessCheckpoint, String> = async {
        let concat = write_concat_file(&directory, &normalized)?;
        let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?
            .ok_or("The bundled FFmpeg tool is unavailable")?;
        let measure_cancellation = cancellation.clone();
        let measurement = crate::blocking::run(move || {
            measure_loudness(
                &ffmpeg,
                &concat,
                config.sample_rate,
                &config,
                &measure_cancellation,
            )
        })
        .await?;
        let encoded = serde_json::to_string(&measurement).map_err(db::json_error)?;
        let inserted = sqlx::query(
            "INSERT INTO spaces_speech_loudness_checkpoints(id,project_revision_id,input_digest,measurement,created_at) \
             SELECT ?,?,?,?,? WHERE EXISTS( \
               SELECT 1 FROM spaces_run_attempts a JOIN spaces_runs r ON r.id=a.run_id \
               WHERE a.id=? AND a.run_id=? AND a.step_instance_id=? AND a.state='executing' \
                 AND a.execution_epoch=? AND a.lease_until>? AND r.execution_epoch=a.execution_epoch \
                 AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(revision_id)
        .bind(&digest)
        .bind(encoded)
        .bind(crate::clock::unix_time_millis()?)
        .bind(&claim.attempt_id)
        .bind(&claim.run_id)
        .bind(&claim.step_instance_id)
        .bind(claim.execution_epoch)
        .bind(crate::clock::unix_time_millis()?)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
        db::require_one(inserted, "attempt-stale: loudness measurement lease expired")?;
        Ok(LoudnessCheckpoint {
            input_digest: digest,
            measurement,
        })
    }
    .await;
    match fs::remove_dir_all(&directory) {
        Ok(()) => normalization,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => normalization,
        Err(error) => match normalization {
            Ok(_) => Err(error.to_string()),
            Err(mut failure) => {
                failure.push_str(&format!(
                    "; failed to remove {}: {error}",
                    directory.display()
                ));
                Err(failure)
            }
        },
    }
}
