use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use std::{fs, path::Path};

use sqlx::SqlitePool;

use super::super::video_config::SpeechMediaSource;
use super::{
    contracts::SpeechSourceSeparationCandidate,
    media::{deadline, extract_audio, hash_file, pcm_wav_metadata, workspace, SAMPLE_RATE},
    repository::{load_candidate, load_candidate_optional, persist_candidate},
    SeparationConfiguration,
};
use crate::{
    ai::{
        contracts::speech_wire::{
            AudioFileReference, InferenceRequestIdentity, SpeechSeparateParams,
        },
        local_runtime::{admission::Priority, separation},
    },
    app_error::AppError,
    local_resources::{preparation, tools},
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
    let config = SeparationConfiguration::from_claim(claim)?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    super::media::require_owned_source(&mut tx, &claim.home_space_id, &config.source).await?;
    tx.rollback().await.map_err(db::sql_error)?;
    let source =
        crate::spaces::assets::authorize_spaces_asset(app, Path::new(&config.source.asset_path))?;
    crate::file_inspection::verify_content_revision(&source, &config.source.source_revision)?;
    if let Some(candidate) = load_candidate_optional(pool, &claim.run_id).await? {
        return resume_completed(app, claim, &config.source, candidate);
    }
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
    let extracted = workspace.join("source.wav");
    let separation = execute_inner(
        app,
        claim,
        &config.source,
        &source,
        &extracted,
        &vocals_part,
        &background_part,
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
    let candidate = persist_candidate(
        pool,
        claim,
        &config.source,
        &background,
        &vocals,
        &metadata,
        duration_ms,
    )
    .await;
    let _ = fs::remove_dir_all(&workspace);
    if let Err(error) = candidate {
        super::super::asset_publication::abandon_attempt(app, pool, claim).await?;
        return Err(error.into());
    }
    completed(load_candidate(pool, &claim.run_id).await?)
}

fn resume_completed(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    source: &SpeechMediaSource,
    candidate: SpeechSourceSeparationCandidate,
) -> Result<StepOutcome, AppError> {
    if !candidate_matches_claim(claim, source, &candidate) {
        return Err("stale_revision: persisted source separation differs from its run".into());
    }
    for stem in [&candidate.background, &candidate.vocals] {
        let path = crate::spaces::assets::authorize_spaces_asset(app, Path::new(&stem.asset_path))?;
        let expected = stem
            .source_revision
            .strip_prefix("sha256:")
            .ok_or("Persisted source separation revision is invalid")?;
        if hash_file(&path, &claim.cancellation, deadline(source.duration_ms)?)? != expected {
            return Err("stale_revision: persisted source separation track changed".into());
        }
    }
    completed(candidate)
}

pub(super) fn candidate_matches_claim(
    claim: &AttemptClaim,
    source: &SpeechMediaSource,
    candidate: &SpeechSourceSeparationCandidate,
) -> bool {
    candidate.space_id == claim.home_space_id
        && candidate.run_id == claim.run_id
        && candidate.separation_id == claim.run_id
        && SeparationConfiguration::from_claim(claim).is_ok_and(|config| {
            config.source == *source
                && config
                    .model
                    .separation_spec()
                    .is_ok_and(|spec| candidate.model_spec_id == spec)
        })
        && candidate.source == *source
}

fn completed(candidate: SpeechSourceSeparationCandidate) -> Result<StepOutcome, AppError> {
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(candidate).map_err(db::json_error)?,
        selected_branch: None,
    })
}

async fn execute_inner(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    source_ref: &SpeechMediaSource,
    source: &Path,
    extracted: &Path,
    vocals: &Path,
    background: &Path,
) -> Result<separation::SeparatedAudio, String> {
    let config = SeparationConfiguration::from_claim(claim)?;
    let device_profile =
        crate::ai::local_runtime::device_profiles::speech_separation(config.model)?;
    let ffmpeg = tools::resolve_ffmpeg(app)?.ok_or("FFmpeg runtime is unavailable")?;
    extract_audio(&ffmpeg, source, extracted, &claim.cancellation)?;
    crate::file_inspection::verify_content_revision(source, &source_ref.source_revision)?;
    let (byte_length, sample_count) = pcm_wav_metadata(extracted)?;
    let hash_deadline = deadline(source_ref.duration_ms)?;
    let sha256 = hash_file(extracted, &claim.cancellation, hash_deadline)?;
    let models = preparation::prepare_models_for_job(
        app,
        config.model.preparation(),
        &claim.attempt_id,
        "Preparing source separation",
        &claim.cancellation,
        &mut |_| {},
    )
    .await?;
    let request_deadline = deadline(source_ref.duration_ms)?;
    let identity = InferenceRequestIdentity {
        request_id: claim.run_id.clone(),
        owner_kind: "speech_source_separation".into(),
        owner_id: claim.run_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        attempt_epoch: u64::try_from(claim.execution_epoch)
            .map_err(|_| "Source separation execution epoch is invalid")?,
        input_revision: source_ref.source_revision.clone(),
        model_spec_id: config.model.separation_spec()?.into(),
        deadline_ms: request_deadline,
    };
    let metadata = separation::separate_to_paths(
        app,
        &models,
        device_profile,
        SpeechSeparateParams {
            identity,
            audio_ref: AudioFileReference {
                path: extracted.to_string_lossy().into_owned(),
                media_type: "audio/wav".into(),
                byte_length,
                sha256,
                sample_rate: SAMPLE_RATE,
                channels: 2,
                sample_format: "pcm_s16le".into(),
            },
        },
        &claim.cancellation,
        Priority::Background,
        separation::SeparationTargets { vocals, background },
    )
    .await?;
    if metadata.background.sample_count != sample_count {
        let _ = fs::remove_file(vocals);
        let _ = fs::remove_file(background);
        return Err("invalid_output: separated background timeline differs from its source".into());
    }
    Ok(metadata)
}
