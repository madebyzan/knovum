use std::{
    fs,
    path::{Path, PathBuf},
};

use sqlx::{Sqlite, SqlitePool, Transaction};
use tauri::{AppHandle, Runtime};

use crate::spaces::{assets, authorization::RunScope, db, runs::AttemptClaim};

#[derive(Clone, Copy)]
pub(crate) enum PublicationOwnerKind {
    SegmentRender,
    ChapterCheckpoint,
    ProjectOutput,
    VoiceReference,
    VoiceCandidate,
    VoiceCandidateInput,
    SourceSeparation,
}
impl PublicationOwnerKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::SegmentRender => "segment_render",
            Self::ChapterCheckpoint => "chapter_checkpoint",
            Self::ProjectOutput => "project_output",
            Self::VoiceReference => "voice_reference",
            Self::VoiceCandidate => "voice_candidate",
            Self::VoiceCandidateInput => "voice_candidate_input",
            Self::SourceSeparation => "source_separation",
        }
    }
}

pub(crate) struct PendingAsset<'a> {
    pub owner_kind: PublicationOwnerKind,
    pub owner_id: &'a str,
    pub asset_path: &'a Path,
    pub part_path: Option<&'a Path>,
    pub sha256: &'a str,
}

pub(crate) async fn stage_for_attempt(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    asset: PendingAsset<'_>,
) -> Result<String, String> {
    Ok(stage_batch_for_attempt(pool, claim, &[asset])
        .await?
        .remove(0))
}

pub(crate) async fn stage_batch_for_attempt(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    assets: &[PendingAsset<'_>],
) -> Result<Vec<String>, String> {
    let mut tx = db::begin_write(pool).await?;
    let mut staged = Vec::with_capacity(assets.len());
    for asset in assets {
        staged.push(stage_in_transaction(&mut tx, claim, asset).await?);
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(staged)
}

async fn stage_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    asset: &PendingAsset<'_>,
) -> Result<String, String> {
    if claim.cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let now = crate::clock::unix_time_millis()?;
    let id = uuid::Uuid::new_v4().to_string();
    let (scope_kind, space_id) = claim.scope.db_values();
    let inserted = sqlx::query(
        "INSERT INTO spaces_speech_asset_publications( \
           id,scope_kind,space_id,owner_kind,owner_id,asset_path,part_path,sha256,run_id,run_step_id, \
           run_attempt_id,run_execution_epoch,created_at) \
         SELECT ?,?,?,?,?,?,?,?,?,?,?,?,? WHERE EXISTS( \
           SELECT 1 FROM spaces_run_attempts a JOIN spaces_runs r ON r.id=a.run_id \
           WHERE a.id=? AND a.run_id=? AND a.step_instance_id=? AND a.state='executing' \
             AND a.execution_epoch=? AND a.lease_until>? AND r.execution_epoch=a.execution_epoch \
             AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL \
             AND r.scope_kind=? AND r.home_space_id IS ?)",
    )
    .bind(&id)
    .bind(scope_kind)
    .bind(space_id)
    .bind(asset.owner_kind.as_str())
    .bind(asset.owner_id)
    .bind(asset.asset_path.to_string_lossy().as_ref())
    .bind(
        asset
            .part_path
            .map(|path| path.to_string_lossy().into_owned()),
    )
    .bind(asset.sha256)
    .bind(&claim.run_id)
    .bind(&claim.step_instance_id)
    .bind(&claim.attempt_id)
    .bind(claim.execution_epoch)
    .bind(now)
    .bind(&claim.attempt_id)
    .bind(&claim.run_id)
    .bind(&claim.step_instance_id)
    .bind(claim.execution_epoch)
    .bind(now)
    .bind(scope_kind)
    .bind(space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        inserted,
        "attempt-stale: asset publication lease is no longer active",
    )?;
    Ok(id)
}

pub(crate) async fn stage_unowned(
    pool: &SqlitePool,
    scope: &RunScope,
    asset: PendingAsset<'_>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let (scope_kind, space_id) = scope.db_values();
    sqlx::query(
        "INSERT INTO spaces_speech_asset_publications( \
           id,scope_kind,space_id,owner_kind,owner_id,asset_path,part_path,sha256,created_at) VALUES(?,?,?,?,?,?,?,?,?)",
    )
    .bind(&id)
    .bind(scope_kind)
    .bind(space_id)
    .bind(asset.owner_kind.as_str())
    .bind(asset.owner_id)
    .bind(asset.asset_path.to_string_lossy().as_ref())
    .bind(
        asset
            .part_path
            .map(|path| path.to_string_lossy().into_owned()),
    )
    .bind(asset.sha256)
    .bind(crate::clock::unix_time_millis()?)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(id)
}

pub(super) async fn seal_unowned(
    pool: &SqlitePool,
    publication_id: &str,
    sha256: &str,
) -> Result<(), String> {
    let sealed = sqlx::query(
        "UPDATE spaces_speech_asset_publications SET sha256=? \
         WHERE id=? AND run_id IS NULL AND sha256=?",
    )
    .bind(sha256)
    .bind(publication_id)
    .bind("0".repeat(64))
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(sealed, "Asset preparation reservation is no longer active")
}

pub(crate) async fn remove_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    publication_id: &str,
) -> Result<(), String> {
    let deleted = sqlx::query("DELETE FROM spaces_speech_asset_publications WHERE id=?")
        .bind(publication_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(deleted, "Asset publication ownership record is missing")
}

pub(super) async fn remove_attempt_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    asset_path: &Path,
    sha256: &str,
) -> Result<(), String> {
    let deleted = sqlx::query(
        "DELETE FROM spaces_speech_asset_publications \
         WHERE run_attempt_id=? AND run_execution_epoch=? AND asset_path=? AND sha256=?",
    )
    .bind(&claim.attempt_id)
    .bind(claim.execution_epoch)
    .bind(asset_path.to_string_lossy().as_ref())
    .bind(sha256)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(deleted, "Asset publication ownership record is missing")
}

pub(crate) async fn abandon<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    publication_id: &str,
) -> Result<(), String> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT asset_path,part_path FROM spaces_speech_asset_publications WHERE id=?",
    )
    .bind(publication_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some((asset_path, part_path)) = row else {
        return Ok(());
    };
    remove_candidate(app, Path::new(&asset_path))?;
    if let Some(path) = part_path {
        remove_candidate(app, Path::new(&path))?;
    }
    sqlx::query("DELETE FROM spaces_speech_asset_publications WHERE id=?")
        .bind(publication_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn abandon_attempt<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<(), String> {
    let publication_ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_speech_asset_publications \
         WHERE run_attempt_id=? AND run_execution_epoch=? ORDER BY created_at,id",
    )
    .bind(&claim.attempt_id)
    .bind(claim.execution_epoch)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for publication_id in publication_ids {
        abandon(app, pool, &publication_id).await?;
    }
    Ok(())
}

pub(super) async fn reconcile<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
) -> Result<(), String> {
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id,asset_path,part_path FROM spaces_speech_asset_publications ORDER BY created_at,id",
    ).fetch_all(pool).await.map_err(db::sql_error)?;
    for (id, asset_path, part_path) in rows {
        remove_candidate(app, Path::new(&asset_path))?;
        if let Some(path) = part_path {
            remove_candidate(app, Path::new(&path))?;
        }
        sqlx::query("DELETE FROM spaces_speech_asset_publications WHERE id=?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

fn remove_candidate<R: Runtime>(app: &AppHandle<R>, path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let root = assets::asset_directory(app)?
                .canonicalize()
                .map_err(|error| error.to_string())?;
            if !path.is_absolute()
                || !path.starts_with(&root)
                || path.components().any(|component| {
                    matches!(
                        component,
                        std::path::Component::ParentDir | std::path::Component::CurDir
                    )
                })
            {
                return Err(
                    "Pending speech asset path is outside the managed asset directory".into(),
                );
            }
            return Ok(());
        }
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Pending speech asset is not a managed regular file".into());
    }
    let path: PathBuf = assets::authorize_spaces_asset(app, path)?;
    fs::remove_file(path).map_err(|error| error.to_string())
}
