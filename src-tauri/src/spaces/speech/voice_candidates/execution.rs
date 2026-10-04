use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use super::{
    artifacts, repository, VoiceCandidateSource, VoiceCandidateStepConfiguration, MODEL_SPEC_ID,
};
use crate::{
    ai::{
        contracts::speech_wire::{
            AudioFileReference, InferenceRequestIdentity, SpeechSynthesisEngine,
            SpeechSynthesizeParams, SynthesisProfile, VoiceRevisionReference, VoiceSourceKind,
        },
        local_runtime::{admission::Priority, speech},
    },
    app_error::AppError,
    local_resources::preparation,
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
    let configuration: VoiceCandidateStepConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let request = &configuration.request;
    if request.request_id != claim.run_id || request.space_id != claim.home_space_id {
        return Err("stale_revision: voice candidate request differs from its Run".into());
    }
    if let Some(record) =
        repository::load_record_optional(pool, &claim.run_id, &claim.home_space_id).await?
    {
        artifacts::validate_persisted(app, claim, &record)?;
        artifacts::cleanup_source_snapshot(app, &configuration)?;
        return completed(record.candidate);
    }
    let paths = artifacts::CandidatePaths::new(app, claim)?;
    paths.require_available()?;
    artifacts::verify_source_snapshot(app, &configuration).await?;
    let result = execute_inner(app, pool, claim, &configuration, &paths).await;
    paths.cleanup_working();
    if result.is_err() {
        let persisted: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM spaces_voice_candidates WHERE id=? AND space_id=?)",
        )
        .bind(&claim.run_id)
        .bind(&claim.home_space_id)
        .fetch_one(pool)
        .await
        .map_err(db::sql_error)?;
        if !persisted {
            super::super::asset_publication::abandon_attempt(app, pool, claim).await?;
            paths.cleanup_published();
        }
    }
    result
}

async fn execute_inner(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
    configuration: &VoiceCandidateStepConfiguration,
    paths: &artifacts::CandidatePaths,
) -> Result<StepOutcome, AppError> {
    let request = &configuration.request;
    let (reference, reference_text, description, source_kind) =
        prepare_source(app, claim, configuration, &paths.reference_part).await?;
    artifacts::verify_source_snapshot(app, configuration).await?;
    let models = preparation::prepare_voxcpm2_for_job(
        app,
        &claim.attempt_id,
        "Preparing voice candidate",
        &claim.cancellation,
        &mut |_| {},
    )
    .await?;
    let deadline_ms: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "Clock exceeds protocol range")?;
    let deadline_ms = deadline_ms
        .checked_add(300_000)
        .ok_or("Voice candidate deadline overflow")?;
    let input = serde_json::to_vec(request).map_err(db::json_error)?;
    let audio = speech::synthesize_voxcpm2(
        app,
        &models,
        crate::ai::local_runtime::device_profiles::voxcpm2()?,
        SpeechSynthesizeParams {
            identity: InferenceRequestIdentity {
                request_id: claim.run_id.clone(),
                owner_kind: "voice_candidate".into(),
                owner_id: claim.home_space_id.clone(),
                attempt_id: claim.attempt_id.clone(),
                attempt_epoch: u64::try_from(claim.execution_epoch)
                    .map_err(|_| "Voice candidate execution epoch is invalid")?,
                input_revision: format!("{:x}", Sha256::digest(input)),
                model_spec_id: MODEL_SPEC_ID.into(),
                deadline_ms,
            },
            text: request.text.trim().into(),
            voice_revision: VoiceRevisionReference {
                revision_id: claim.run_id.clone(),
                source_kind: if source_kind == "reference" {
                    VoiceSourceKind::Reference
                } else {
                    VoiceSourceKind::Designed
                },
                speaker_id: None,
                reference_audio: reference.clone(),
                reference_text: reference_text.clone(),
                design_prompt: description.clone(),
            },
            style: None,
            seed: 0,
            profile: SynthesisProfile {
                engine: SpeechSynthesisEngine::VoxCpm2,
                language: request.language.clone(),
                speed: 1.0,
                sample_rate: 48_000,
            },
        },
        speech::SynthesisExecution {
            cancellation: &claim.cancellation,
            priority: Priority::Interactive,
            on_chunk: None,
        },
    )
    .await?;
    super::super::publication::validate_synthesized_audio(&audio)?;
    artifacts::write_new(&paths.preview_part, &audio.bytes, &claim.cancellation)?;
    let reference = if source_kind == "designed" {
        super::super::voice_assets::prepare_reference(
            app,
            &paths.preview_part,
            &paths.reference_part,
            u64::try_from(audio.bytes.len())
                .map_err(|_| "Voice candidate preview exceeds this platform")?,
            &claim.cancellation,
        )?
    } else {
        reference.ok_or("Voice clone reference was not prepared")?
    };
    let preview_sha = super::super::publication::hash_file(&paths.preview_part)?;
    if preview_sha != audio.sha256 {
        return Err("invalid_output: voice candidate preview digest differs".into());
    }
    artifacts::stage(
        pool,
        claim,
        &paths.preview,
        &paths.preview_part,
        &preview_sha,
    )
    .await?;
    artifacts::stage(
        pool,
        claim,
        &paths.reference,
        &paths.reference_part,
        &reference.sha256,
    )
    .await?;
    fs::rename(&paths.preview_part, &paths.preview).map_err(|error| error.to_string())?;
    fs::rename(&paths.reference_part, &paths.reference).map_err(|error| error.to_string())?;
    repository::persist_candidate(
        pool,
        claim,
        configuration,
        source_kind,
        description.as_deref(),
        reference_text.as_deref(),
        &paths.preview,
        &preview_sha,
        audio.sample_rate,
        audio.sample_count,
        &paths.reference,
        &reference,
    )
    .await?;
    artifacts::cleanup_source_snapshot(app, configuration)?;
    completed(repository::load_candidate(pool, &claim.run_id, &claim.home_space_id).await?)
}

async fn prepare_source(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    configuration: &VoiceCandidateStepConfiguration,
    reference_part: &Path,
) -> Result<
    (
        Option<AudioFileReference>,
        Option<String>,
        Option<String>,
        &'static str,
    ),
    AppError,
> {
    let request = &configuration.request;
    match &request.source {
        VoiceCandidateSource::Designed { description } => Ok((
            None,
            Some(request.text.trim().into()),
            Some(description.trim().into()),
            "designed",
        )),
        VoiceCandidateSource::Reference {
            source_path,
            reference_text,
            ..
        } => {
            let snapshot = configuration
                .reference_source
                .as_ref()
                .ok_or("Voice candidate reference snapshot is missing")?;
            let audio = super::super::voice_assets::prepare_reference(
                app,
                Path::new(source_path),
                reference_part,
                snapshot.byte_length,
                &claim.cancellation,
            )?;
            let text = reference_text
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty());
            if let Err(error) = super::super::voice_reference_validation::validate(
                app,
                &claim.run_id,
                &claim.home_space_id,
                &request.language,
                audio.clone(),
                text,
                &claim.cancellation,
            )
            .await
            {
                let _ = fs::remove_file(reference_part);
                return Err(error.into());
            }
            Ok((Some(audio), text.map(str::to_owned), None, "reference"))
        }
    }
}

fn completed(candidate: super::VoiceCandidate) -> Result<StepOutcome, AppError> {
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(candidate).map_err(db::json_error)?,
        selected_branch: None,
    })
}
