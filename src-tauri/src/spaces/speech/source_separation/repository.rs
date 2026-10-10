use std::path::Path;

use sqlx::{Row, SqlitePool};

use super::{contracts::SpeechSourceSeparationPublication, SeparationConfiguration};
use crate::{
    ai::local_runtime::separation,
    spaces::{db, runs::AttemptClaim, speech::video_config::SpeechMediaSource},
};

pub(super) async fn load_publication(
    pool: &SqlitePool,
    separation_id: &str,
) -> Result<SpeechSourceSeparationPublication, String> {
    find_publication(pool, separation_id)
        .await?
        .ok_or("Source separation result is unavailable".into())
}

macro_rules! publication_query {
    ($predicate:literal) => {
        concat!("SELECT id,space_id,run_id,source_node_id,source_asset_path,source_revision,source_duration_ms,model_spec_id,background_asset_path,background_sha256,vocals_asset_path,vocals_sha256,background_duration_ms FROM spaces_speech_source_separations ", $predicate)
    };
}

pub(super) async fn find_publication(
    pool: &SqlitePool,
    separation_id: &str,
) -> Result<Option<SpeechSourceSeparationPublication>, String> {
    sqlx::query(publication_query!("WHERE id=?"))
        .bind(separation_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .map(decode_publication)
        .transpose()
}

pub(super) async fn list_publications_for_source(
    pool: &SqlitePool,
    space_id: &str,
    source: &SpeechMediaSource,
) -> Result<Vec<SpeechSourceSeparationPublication>, String> {
    sqlx::query(publication_query!("WHERE space_id=? AND source_asset_path=? AND source_revision=? AND source_duration_ms=? ORDER BY created_at DESC,id DESC"))
        .bind(space_id).bind(&source.asset_path).bind(&source.source_revision)
        .bind(i64::try_from(source.duration_ms).map_err(|_| "Media duration is too large")?)
        .fetch_all(pool).await.map_err(db::sql_error)?.into_iter().map(decode_publication).collect()
}

fn decode_publication(
    row: sqlx::sqlite::SqliteRow,
) -> Result<SpeechSourceSeparationPublication, String> {
    let duration = |column: &str| -> Result<u64, String> {
        u64::try_from(row.try_get::<i64, _>(column).map_err(db::sql_error)?)
            .map_err(|_| "Source separation duration is invalid".into())
    };
    Ok(SpeechSourceSeparationPublication {
        separation_id: row.try_get("id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        run_id: row.try_get("run_id").map_err(db::sql_error)?,
        source_node_id: row.try_get("source_node_id").map_err(db::sql_error)?,
        source: SpeechMediaSource {
            asset_path: row.try_get("source_asset_path").map_err(db::sql_error)?,
            source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
            duration_ms: duration("source_duration_ms")?,
        },
        background: SpeechMediaSource {
            asset_path: row
                .try_get("background_asset_path")
                .map_err(db::sql_error)?,
            source_revision: format!(
                "sha256:{}",
                row.try_get::<String, _>("background_sha256")
                    .map_err(db::sql_error)?
            ),
            duration_ms: duration("background_duration_ms")?,
        },
        vocals: SpeechMediaSource {
            asset_path: row.try_get("vocals_asset_path").map_err(db::sql_error)?,
            source_revision: format!(
                "sha256:{}",
                row.try_get::<String, _>("vocals_sha256")
                    .map_err(db::sql_error)?
            ),
            duration_ms: duration("background_duration_ms")?,
        },
        model_spec_id: row.try_get("model_spec_id").map_err(db::sql_error)?,
    })
}

pub(super) async fn commit_publication(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    source: &SpeechMediaSource,
    background: &Path,
    vocals: &Path,
    metadata: &separation::SeparatedAudio,
    duration_ms: u64,
) -> Result<(), String> {
    if claim.cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let space_id = claim.scope.require_space_id()?;
    let config = SeparationConfiguration::from_claim(claim)?;
    if config.source != *source {
        return Err(
            "stale_revision: separation source differs from its frozen configuration".into(),
        );
    }
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    let is_attempt_active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_run_attempts a JOIN spaces_runs r ON r.id=a.run_id \
         WHERE a.id=? AND a.run_id=? AND a.step_instance_id=? AND a.state='executing' \
           AND a.execution_epoch=? AND a.lease_until>? AND r.execution_epoch=a.execution_epoch \
           AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL \
           AND r.scope_kind='space' AND r.home_space_id=?)",
    )
    .bind(&claim.attempt_id)
    .bind(&claim.run_id)
    .bind(&claim.step_instance_id)
    .bind(claim.execution_epoch)
    .bind(now)
    .bind(space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if !is_attempt_active {
        return Err("attempt-stale: source separation lease is no longer active".into());
    }
    sqlx::query(
        "INSERT INTO spaces_speech_source_separations( \
           id,space_id,run_id,source_node_id,source_asset_path,source_revision,source_duration_ms,model_spec_id, \
           background_asset_path,background_sha256,vocals_asset_path,vocals_sha256,background_sample_rate,background_sample_count, \
           background_duration_ms,created_at) \
         VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&claim.run_id)
    .bind(space_id)
    .bind(&claim.run_id)
    .bind(&config.source_node_id)
    .bind(&source.asset_path)
    .bind(&source.source_revision)
    .bind(i64::try_from(source.duration_ms).map_err(|_| "Source duration exceeds SQLite range")?)
    .bind(config.model.separation_spec()?)
    .bind(background.to_string_lossy().as_ref())
    .bind(&metadata.background.sha256)
    .bind(vocals.to_string_lossy().as_ref())
    .bind(&metadata.vocals.sha256)
    .bind(i64::from(metadata.background.sample_rate))
    .bind(i64::try_from(metadata.background.sample_count).map_err(|_| "Sample count exceeds SQLite range")?)
    .bind(i64::try_from(duration_ms).map_err(|_| "Background duration exceeds SQLite range")?)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    super::publication::publish(&mut tx, claim, &config, background, vocals, metadata).await?;
    for (path, digest) in [
        (background, &metadata.background.sha256),
        (vocals, &metadata.vocals.sha256),
    ] {
        super::super::asset_publication::remove_attempt_in_transaction(
            &mut tx, claim, path, digest,
        )
        .await?;
    }
    if claim.cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    tx.commit().await.map_err(db::sql_error)
}
