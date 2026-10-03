use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use tauri::ipc::Channel;

use super::{cancellation, contracts::*, generation_attempt, project_publication, publication};
use crate::{
    ai::{
        contracts::{
            runtime::RuntimeDeviceProfile,
            speech_wire::{
                InferenceRequestIdentity, SpeechSynthesisEngine, SpeechSynthesizeParams,
                SynthesisProfile, VoiceRevisionReference, VoiceSourceKind,
            },
        },
        local_runtime::{admission::Priority, speech},
    },
    app_error::AppError,
    local_resources::preparation,
    spaces::db,
};

pub(crate) const KOKORO_MODEL_SPEC_ID: &str = "kokoro-int8-multi-lang-v1_0";

struct SynthesisAttempt<'a> {
    request_id: &'a str,
    owner_kind: &'a str,
    owner_id: &'a str,
    input_revision: &'a str,
    attempt_id: &'a str,
    attempt_epoch: u64,
}

#[tauri::command]
pub async fn preview_preset_voice(
    app: tauri::AppHandle,
    request: PreviewVoiceRequest,
    on_progress: Channel<SpeechGenerationProgress>,
) -> Result<SpeechAudioOutput, AppError> {
    let lease = cancellation::acquire(&request.job_id, cancellation::JobKind::Preview)?;
    let voice = load_voice(
        db::pool(&app).await?,
        &request.voice_revision_id,
        &request.space_id,
    )
    .await?;
    let text = request.text.trim();
    if text.is_empty() || text.chars().count() > 500 {
        return Err("Voice preview text must contain between 1 and 500 characters".into());
    }
    send(&on_progress, "preparing", 0, 1, None);
    let model = preparation::prepare_kokoro_for_job(
        &app,
        &request.job_id,
        "Preparing preset voice",
        lease.token(),
        &mut |resources| {
            send(&on_progress, "preparing", 0, 1, Some(resources));
        },
    )
    .await
    .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
    let input_revision = digest_text(text);
    let params = synthesis_params(
        &voice,
        text,
        SynthesisAttempt {
            request_id: &request.job_id,
            owner_kind: "speech_preview",
            owner_id: &voice.revision_id,
            input_revision: &input_revision,
            attempt_id: &request.job_id,
            attempt_epoch: 0,
        },
    )?;
    send(&on_progress, "synthesizing", 0, 1, None);
    let audio = speech::synthesize(
        &app,
        &model,
        device_profile(),
        params,
        lease.token(),
        Priority::Interactive,
    )
    .await
    .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
    if lease.token().is_cancelled() {
        return Err(AppError::cancelled("Speech preview cancelled"));
    }
    if audio.sample_rate != parse_config(&voice.synthesis_config)?.sample_rate {
        return Err(
            "invalid_output: synthesized sample rate differs from the voice revision".into(),
        );
    }
    let output = publication::write_preview(&app, &request.job_id, audio)?;
    if lease.token().is_cancelled() {
        publication::discard_preview(&app, &output.path)?;
        return Err(AppError::cancelled("Speech preview cancelled"));
    }
    send(&on_progress, "done", 1, 1, None);
    Ok(output)
}

#[tauri::command]
pub async fn generate_speech_project(
    app: tauri::AppHandle,
    request: GenerateSpeechProjectRequest,
    on_progress: Channel<SpeechGenerationProgress>,
) -> Result<SpeechProjectRenderOutput, AppError> {
    let lease = cancellation::acquire(&request.job_id, cancellation::JobKind::Project)?;
    let pool = db::pool(&app).await?;
    let generation_epoch =
        generation_attempt::begin_generation(pool, &request.project_id, &request.job_id).await?;
    let result =
        generate_speech_project_inner(&app, pool, &request, on_progress, &lease, generation_epoch)
            .await;
    cancellation::require_persisted_cancel(&request.job_id)?;
    let finish = generation_attempt::finish_generation(
        pool,
        &request.project_id,
        &request.job_id,
        generation_epoch,
    )
    .await;
    match (result, finish) {
        (Ok(output), Ok(())) => Ok(output),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
    }
}

async fn generate_speech_project_inner(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    request: &GenerateSpeechProjectRequest,
    on_progress: Channel<SpeechGenerationProgress>,
    lease: &cancellation::JobLease,
    generation_epoch: i64,
) -> Result<SpeechProjectRenderOutput, AppError> {
    let (revision_id, revision, project_type, space_id) =
        active_revision(pool, &request.project_id).await?;
    if revision != request.expected_revision {
        return Err("stale_revision: speech project changed before generation".into());
    }
    if project_type != "short_dubbing" {
        return Err("This speech project requires its dedicated long-form workflow".into());
    }
    let attempt = generation_attempt::GenerationAttempt {
        project_id: &request.project_id,
        revision_id: &revision_id,
        job_id: &request.job_id,
        epoch: generation_epoch,
        cancellation: lease.token(),
    };
    let segments: Vec<SpeechSegmentExecution> = sqlx::query_as(
        "SELECT id,normalized_text,voice_revision_id,language FROM spaces_speech_segments \
         WHERE project_revision_id=? ORDER BY ordinal",
    )
    .bind(&revision_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    if segments.is_empty() {
        return Err("Speech project has no segments to generate".into());
    }
    let total =
        u32::try_from(segments.len()).map_err(|_| "Speech project has too many segments")?;
    let model = preparation::prepare_kokoro_for_job(
        app,
        &request.job_id,
        "Preparing speech synthesis",
        lease.token(),
        &mut |resources| send(&on_progress, "preparing", 0, total, Some(resources)),
    )
    .await
    .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
    let mut renders = Vec::with_capacity(segments.len());
    for (index, segment) in segments.iter().enumerate() {
        if lease.token().is_cancelled() {
            return Err(AppError::cancelled("Speech generation cancelled"));
        }
        let voice = load_voice(pool, &segment.voice_revision_id, &space_id).await?;
        if voice.language != segment.language {
            return Err("Speech segment language differs from its voice revision".into());
        }
        let config = parse_config(&voice.synthesis_config)?;
        let digest = input_digest(segment, &voice, &config)?;
        if let Some(cached) =
            generation_attempt::cached_render(pool, &request.project_id, &segment.id, &digest)
                .await?
        {
            let output = generation_attempt::cached_output(segment, &cached)?;
            let output = publication::validate_cached(app, output)?;
            if lease.token().is_cancelled() {
                return Err(AppError::cancelled("Speech generation cancelled"));
            }
            if cached.segment_id != segment.id {
                generation_attempt::link_cached_render(
                    pool, attempt, segment, &voice, &digest, &cached, false,
                )
                .await
                .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
            } else {
                generation_attempt::mark_cached_segment(pool, attempt, &segment.id, false)
                    .await
                    .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
            }
            renders.push(output);
            send(&on_progress, "rendering", index as u32 + 1, total, None);
            continue;
        }
        let target_path = publication::segment_path(
            app,
            &request.project_id,
            &revision_id,
            &segment.id,
            &digest,
        )?;
        let (render_id, attempt_epoch) =
            generation_attempt::stage_render(pool, attempt, segment, &voice, &digest, &target_path)
                .await?;
        let params = synthesis_params(
            &voice,
            &segment.normalized_text,
            SynthesisAttempt {
                request_id: &request.job_id,
                owner_kind: "speech_project_segment",
                owner_id: &segment.id,
                input_revision: &digest,
                attempt_id: &render_id,
                attempt_epoch,
            },
        )?;
        let audio = match speech::synthesize(
            app,
            &model,
            device_profile(),
            params,
            lease.token(),
            Priority::Background,
        )
        .await
        {
            Ok(audio) => audio,
            Err(error) => {
                generation_attempt::fail_render(
                    pool,
                    &render_id,
                    &segment.id,
                    &request.job_id,
                    generation_epoch,
                    &error,
                    lease.token().is_cancelled(),
                )
                .await?;
                return Err(command_error(error, lease.token().is_cancelled()));
            }
        };
        if lease.token().is_cancelled() {
            generation_attempt::fail_render(
                pool,
                &render_id,
                &segment.id,
                &request.job_id,
                generation_epoch,
                "cancelled",
                true,
            )
            .await?;
            return Err(AppError::cancelled("Speech generation cancelled"));
        }
        if audio.sample_rate != config.sample_rate {
            generation_attempt::fail_render(
                pool,
                &render_id,
                &segment.id,
                &request.job_id,
                generation_epoch,
                "invalid_output: synthesized sample rate differs",
                false,
            )
            .await?;
            return Err(
                "invalid_output: synthesized sample rate differs from the voice revision".into(),
            );
        }
        let path = match publication::publish_segment(&target_path, &audio) {
            Ok(path) => path,
            Err(error) => {
                generation_attempt::fail_render(
                    pool,
                    &render_id,
                    &segment.id,
                    &request.job_id,
                    generation_epoch,
                    &error,
                    false,
                )
                .await?;
                return Err(error.into());
            }
        };
        if let Err(error) =
            publication::commit_render(pool, attempt, &render_id, &segment.id, &path, &audio, false)
                .await
        {
            publication::discard_published_file(&path);
            generation_attempt::fail_render(
                pool,
                &render_id,
                &segment.id,
                &request.job_id,
                generation_epoch,
                &error,
                lease.token().is_cancelled(),
            )
            .await?;
            return Err(command_error(error, lease.token().is_cancelled()));
        }
        renders.push(SpeechSegmentRenderOutput {
            segment_id: segment.id.clone(),
            path: path.to_string_lossy().into_owned(),
            sha256: audio.sha256,
            sample_rate: audio.sample_rate,
            sample_count: audio.sample_count,
            cached: false,
        });
        send(&on_progress, "rendering", index as u32 + 1, total, None);
    }
    cancellation::require_persisted_cancel(&request.job_id)?;
    let output = project_publication::publish_project(
        app,
        pool,
        &request.project_id,
        &revision_id,
        &renders,
    )
    .await?;
    project_publication::commit_project_output(
        pool,
        &request.project_id,
        &revision_id,
        &request.job_id,
        generation_epoch,
        &output,
        lease.token(),
    )
    .await
    .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
    send(&on_progress, "done", total, total, None);
    Ok(SpeechProjectRenderOutput {
        project_id: request.project_id.clone(),
        revision,
        renders,
        output,
    })
}

#[tauri::command]
pub async fn cancel_speech_generation(
    app: tauri::AppHandle,
    job_id: String,
) -> Result<bool, AppError> {
    uuid::Uuid::parse_str(&job_id).map_err(|_| "Speech job identity is invalid")?;
    let pool = db::pool(&app).await?;
    let cancelled = sqlx::query(
        "UPDATE spaces_speech_projects SET generation_cancel_requested_at=? \
         WHERE active_generation_job_id=? AND generation_cancel_requested_at IS NULL",
    )
    .bind(crate::clock::unix_time_millis()?)
    .bind(&job_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?
    .rows_affected()
        > 0;
    Ok(cancellation::cancel(&job_id, cancelled)?)
}

#[tauri::command]
pub fn discard_speech_preview(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    Ok(publication::discard_preview(&app, &path)?)
}

fn synthesis_params(
    voice: &PresetVoiceExecution,
    text: &str,
    attempt: SynthesisAttempt<'_>,
) -> Result<SpeechSynthesizeParams, String> {
    let config = parse_config(&voice.synthesis_config)?;
    Ok(SpeechSynthesizeParams {
        identity: InferenceRequestIdentity {
            request_id: attempt.request_id.to_owned(),
            owner_kind: attempt.owner_kind.to_owned(),
            owner_id: attempt.owner_id.to_owned(),
            attempt_id: attempt.attempt_id.to_owned(),
            attempt_epoch: attempt.attempt_epoch,
            input_revision: attempt.input_revision.to_owned(),
            model_spec_id: voice.model_spec_id.clone(),
            deadline_ms: deadline(300)?,
        },
        text: text.to_owned(),
        voice_revision: VoiceRevisionReference {
            revision_id: voice.revision_id.clone(),
            source_kind: VoiceSourceKind::Preset,
            speaker_id: Some(
                voice
                    .preset_voice_id
                    .parse()
                    .map_err(|_| "Preset speaker identity is invalid")?,
            ),
            reference_audio: None,
            reference_text: None,
            design_prompt: None,
        },
        style: None,
        seed: 0,
        profile: SynthesisProfile {
            engine: SpeechSynthesisEngine::Kokoro,
            language: voice.language.clone(),
            speed: config.speed,
            sample_rate: config.sample_rate,
        },
    })
}

fn parse_config(value: &str) -> Result<PresetSynthesisConfig, String> {
    let value = serde_json::from_str(value).map_err(db::json_error)?;
    parse_synthesis_config(&value)
}

fn command_error(error: String, cancelled: bool) -> AppError {
    if cancelled {
        AppError::cancelled("Speech generation cancelled")
    } else {
        error.into()
    }
}

fn device_profile() -> RuntimeDeviceProfile {
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(4) as u8)
        .unwrap_or(1);
    RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: format!("cpu-int8-threads{threads}-v1"),
        cpu_threads: threads,
    }
}

fn deadline(seconds: u64) -> Result<u64, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    let duration_ms = seconds
        .checked_mul(1_000)
        .ok_or("Speech deadline overflow")?;
    u64::try_from(now.as_millis())
        .map_err(|_| "System time exceeds protocol range".to_owned())?
        .checked_add(duration_ms)
        .ok_or("Speech deadline overflow".to_owned())
}

fn digest_text(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn send(
    channel: &Channel<SpeechGenerationProgress>,
    stage: &str,
    completed: u32,
    total: u32,
    resources: Option<preparation::contracts::ResourcePreparationProgress>,
) {
    let _ = channel.send(SpeechGenerationProgress {
        stage: stage.into(),
        completed,
        total,
        resources,
    });
}

async fn load_voice(
    pool: &sqlx::SqlitePool,
    revision_id: &str,
    space_id: &str,
) -> Result<PresetVoiceExecution, String> {
    let voice: PresetVoiceExecution = sqlx::query_as(
        "SELECT r.id AS revision_id,r.engine_id,r.model_spec_id,r.language,r.preset_voice_id, \
         r.synthesis_config FROM spaces_voice_revisions r \
         JOIN spaces_voice_profiles p ON p.id=r.profile_id \
         WHERE r.id=? AND p.space_id=? AND p.source_kind='preset'",
    )
    .bind(revision_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Preset voice revision is unavailable")?;
    if voice.engine_id != "kokoro" || voice.model_spec_id != KOKORO_MODEL_SPEC_ID {
        return Err("Voice revision does not match the active Kokoro specification".to_owned());
    }
    Ok(voice)
}

async fn active_revision(
    pool: &sqlx::SqlitePool,
    project_id: &str,
) -> Result<(String, i64, String, String), String> {
    sqlx::query_as(
        "SELECT r.id,r.revision,p.project_type,p.space_id FROM spaces_speech_projects p \
         JOIN spaces_speech_project_revisions r ON r.id=p.active_revision_id WHERE p.id=?",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Speech project no longer exists".to_owned())
}

fn input_digest(
    segment: &SpeechSegmentExecution,
    voice: &PresetVoiceExecution,
    config: &PresetSynthesisConfig,
) -> Result<String, String> {
    let value = serde_json::json!({
        "text": segment.normalized_text,
        "voiceRevisionId": voice.revision_id,
        "language": segment.language,
        "modelSpecId": voice.model_spec_id,
        "engineId": voice.engine_id,
        "engineVersion": env!("CARGO_PKG_VERSION"),
        "config": config,
        "seed": 0,
        "preprocessingVersion": 1,
    });
    let bytes = serde_json::to_vec(&value).map_err(db::json_error)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
