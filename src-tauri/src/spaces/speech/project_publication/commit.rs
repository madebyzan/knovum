use super::encoding::wav_sample_count;
use crate::spaces::{
    assets, db,
    runs::AttemptClaim,
    speech::{contracts::SpeechAudioOutput, publication::hash_file},
};
use std::{fs, path::Path};
use tauri::{AppHandle, Runtime};

#[derive(sqlx::FromRow)]
struct StoredProjectOutput {
    output_asset_path: Option<String>,
    output_sha256: Option<String>,
    output_sample_rate: Option<i64>,
    output_sample_count: Option<i64>,
}

pub(super) struct CommittedOutputSpec<'a> {
    pub(super) project_id: &'a str,
    pub(super) revision_id: &'a str,
    pub(super) digest: &'a str,
    pub(super) format: &'a str,
    pub(super) sample_rate: u32,
    pub(super) wav_precision: crate::spaces::speech::audio_format::WavPrecision,
}

pub(super) async fn reusable_committed_output<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    expected: CommittedOutputSpec<'_>,
) -> Result<Option<SpeechAudioOutput>, String> {
    let CommittedOutputSpec {
        project_id,
        revision_id,
        digest,
        format,
        sample_rate,
        wav_precision,
    } = expected;
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
        let Ok(actual_samples) = wav_sample_count(&canonical, sample_rate, wav_precision) else {
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
        crate::spaces::speech::asset_publication::remove_attempt_in_transaction(
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
