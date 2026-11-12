use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use std::{fs, path::Path};

use sqlx::SqlitePool;

use super::super::video_config::SpeechMediaSource;
use super::{
    contracts::SpeechSourceSeparationPublication,
    media::{budget_ms, hash_file, workspace},
    repository::{commit_publication, find_publication, load_publication},
    SeparationConfiguration,
};
use crate::{
    ai::local_runtime::separation::{SeparatedAudio, SeparationTargets},
    foundation::app_error::AppError,
    spaces::{
        db,
        runs::{AttemptClaim, StepOutcome},
    },
};

pub(crate) async fn execute(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    super::super::run_execution::require_authority(pool, claim).await?;
    let space_id = claim.scope.require_space_id()?;
    let config = SeparationConfiguration::from_claim(claim)?;
    if let Some(publication) = find_publication(pool, &claim.run_id).await? {
        return resume_completed(app, claim, space_id, &config.source, publication);
    }
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    super::media::require_owned_source(&mut tx, space_id, &config.source_node_id, &config.source)
        .await?;
    tx.rollback().await.map_err(db::sql_error)?;
    let source =
        crate::spaces::assets::authorize_spaces_asset(app, Path::new(&config.source.asset_path))?;
    crate::file_inspection::verify_content_revision(&source, &config.source.source_revision)?;
    let output_directory = crate::spaces::assets::asset_directory(app)?
        .join("speech")
        .join("source-separation");
    fs::create_dir_all(&output_directory).map_err(|error| error.to_string())?;
    let background = output_directory.join(format!("{}-background.wav", claim.run_id));
    let background_part =
        output_directory.join(format!(".{}-background.wav.part", claim.attempt_id));
    let vocals = output_directory.join(format!("{}-vocals.wav", claim.run_id));
    let vocals_part = output_directory.join(format!(".{}-vocals.wav.part", claim.attempt_id));
    for path in [&background, &background_part, &vocals, &vocals_part] {
        if path.exists() {
            return Err("Source separation output path is already occupied".into());
        }
    }
    let workspace = workspace(app, &claim.attempt_id)?;
    let separation = execute_inner(
        app,
        claim,
        &config.source,
        &source,
        &workspace,
        SeparationTargets {
            vocals: &vocals_part,
            background: &background_part,
        },
    )
    .await;
    let metadata = match separation {
        Ok(metadata) => metadata,
        Err(error) => {
            let _ = fs::remove_dir_all(&workspace);
            let _ = fs::remove_file(&background_part);
            let _ = fs::remove_file(&vocals_part);
            return Err(error.into());
        }
    };
    let pending = [
        super::super::asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::SourceSeparation,
            owner_id: &claim.run_id,
            asset_path: &background,
            part_path: Some(&background_part),
            sha256: &metadata.background.sha256,
        },
        super::super::asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::SourceSeparation,
            owner_id: &claim.run_id,
            asset_path: &vocals,
            part_path: Some(&vocals_part),
            sha256: &metadata.vocals.sha256,
        },
    ];
    if let Err(error) =
        super::super::asset_publication::stage_batch_for_attempt(pool, claim, &pending).await
    {
        let _ = fs::remove_dir_all(&workspace);
        let _ = fs::remove_file(&background_part);
        let _ = fs::remove_file(&vocals_part);
        return Err(error.into());
    }
    for (part, output) in [(&background_part, &background), (&vocals_part, &vocals)] {
        if let Err(error) = fs::rename(part, output) {
            super::super::asset_publication::abandon_attempt(app, pool, claim).await?;
            let _ = fs::remove_dir_all(&workspace);
            return Err(error.to_string().into());
        }
    }
    let duration_ms = metadata
        .background
        .sample_count
        .checked_mul(1_000)
        .ok_or("Separated background duration overflow")?
        .div_ceil(u64::from(metadata.background.sample_rate));
    let publication_commit = commit_publication(
        pool,
        claim,
        &config.source,
        &background,
        &vocals,
        &metadata,
        duration_ms,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await;
    let _ = fs::remove_dir_all(&workspace);
    if let Err(error) = publication_commit {
        super::super::asset_publication::abandon_attempt(app, pool, claim).await?;
        return Err(error.into());
    }
    completed(load_publication(pool, &claim.run_id).await?)
}

fn resume_completed(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    space_id: &str,
    source: &SpeechMediaSource,
    publication: SpeechSourceSeparationPublication,
) -> Result<StepOutcome, AppError> {
    if !is_publication_for_claim(claim, space_id, source, &publication) {
        return Err("stale_revision: persisted source separation differs from its run".into());
    }
    for stem in [&publication.background, &publication.vocals] {
        let path = crate::spaces::assets::authorize_spaces_asset(app, Path::new(&stem.asset_path))?;
        let expected = stem
            .source_revision
            .strip_prefix("sha256:")
            .ok_or("Persisted source separation revision is invalid")?;
        if hash_file(
            &path,
            &claim.cancellation,
            hash_deadline(source.duration_ms),
        )? != expected
        {
            return Err("stale_revision: persisted source separation track changed".into());
        }
    }
    completed(publication)
}

pub(super) fn is_publication_for_claim(
    claim: &AttemptClaim,
    space_id: &str,
    source: &SpeechMediaSource,
    publication: &SpeechSourceSeparationPublication,
) -> bool {
    publication.space_id == space_id
        && publication.run_id == claim.run_id
        && publication.separation_id == claim.run_id
        && SeparationConfiguration::from_claim(claim).is_ok_and(|config| {
            config.source == *source
                && config.source_node_id == publication.source_node_id
                && config
                    .model
                    .separation_spec()
                    .is_ok_and(|spec| publication.model_spec_id == spec)
        })
        && publication.source == *source
}

fn completed(publication: SpeechSourceSeparationPublication) -> Result<StepOutcome, AppError> {
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(publication).map_err(db::json_error)?,
        selected_branch: None,
    })
}

async fn execute_inner(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    source: &SpeechMediaSource,
    path: &Path,
    workspace: &Path,
    targets: SeparationTargets<'_>,
) -> Result<SeparatedAudio, String> {
    super::separate::separate(super::separate::Separation {
        app,
        model: SeparationConfiguration::from_claim(claim)?.model,
        path,
        source,
        requester: super::separate::Requester {
            request_id: &claim.run_id,
            owner_kind: "speech_source_separation",
            owner_id: &claim.run_id,
            attempt_id: &claim.attempt_id,
            attempt_epoch: u64::try_from(claim.execution_epoch)
                .map_err(|_| "Source separation execution epoch is invalid")?,
        },
        workspace,
        targets,
        cancellation: &claim.cancellation,
    })
    .await
}

/// Hashing the staged audio is local I/O, so it shares the separation budget, started where the
/// read begins rather than at an absolute instant decided earlier.
fn hash_deadline(duration_ms: u64) -> std::time::Instant {
    std::time::Instant::now() + std::time::Duration::from_millis(budget_ms(duration_ms))
}
