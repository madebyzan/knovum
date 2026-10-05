pub(super) use voice::{input_digest, load_voice};
mod voice;
pub(super) use request::{synthesis_params, SynthesisAttempt};
mod request;

use sha2::{Digest, Sha256};
use tauri::ipc::Channel;

use super::{cancellation, contracts::*, publication, run_start};
use crate::{
    ai::{
        contracts::runtime::RuntimeDeviceProfile,
        local_runtime::{admission::Priority, speech},
    },
    app_error::AppError,
    local_resources::preparation,
    spaces::{db, runs::RunObservation},
};

pub(crate) const KOKORO_MODEL_SPEC_ID: &str = "kokoro-int8-multi-lang-v1_0";
// Preview packet sequence is a bounded IPC transport sequence. The runtime validates the
// originating model-chunk or sentence sequence before this presentation-only split.
const PREVIEW_AUDIO_PACKET_BYTES: usize = 128 * 1024;

#[tauri::command]
pub async fn preview_voice(
    app: tauri::AppHandle,
    request: PreviewVoiceRequest,
    on_progress: Channel<SpeechGenerationProgress>,
    on_audio_chunk: Channel<SpeechPreviewAudioChunk>,
) -> Result<SpeechAudioOutput, AppError> {
    let lease = cancellation::acquire(&request.job_id)?;
    let pool = db::pool(&app).await?;
    let voice = match &request.source {
        VoicePreviewSource::Revision { revision_id } => {
            load_voice(pool, revision_id, &request.space_id).await?
        }
        VoicePreviewSource::Preset {
            speaker_id,
            language,
        } => {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces WHERE id=?)")
                .bind(&request.space_id)
                .fetch_one(pool)
                .await
                .map_err(db::sql_error)?;
            if !exists {
                return Err("Voice preview Space is unavailable".into());
            }
            super::preset_voices::require(speaker_id, language)?;
            VoiceExecution {
                revision_id: format!("kokoro-preset:{speaker_id}"),
                source_kind: "preset".into(),
                engine_id: "kokoro".into(),
                model_spec_id: KOKORO_MODEL_SPEC_ID.into(),
                language: language.clone(),
                preset_voice_id: Some(speaker_id.clone()),
                reference_asset_id: None,
                reference_source_revision: None,
                reference_text: None,
                design_description: None,
                synthesis_config: serde_json::json!({ "speed": 1, "sampleRate": 24000, "seed": 0 })
                    .to_string(),
            }
        }
    };
    let text = request.text.trim();
    if text.is_empty() || text.chars().count() > 500 {
        return Err("Voice preview text must contain between 1 and 500 characters".into());
    }
    let lexicon = super::text::parse_lexicon(&request.pronunciation_lexicon)?;
    let normalized = super::text::apply_lexicon(text, &lexicon);
    if normalized.chars().count() > 500 {
        return Err("Pronunciation substitutions exceed the preview limit".into());
    }
    send(&on_progress, "preparing", 0, 1, None);
    let input_revision = digest_text(&normalized);
    let params = synthesis_params(
        &app,
        &voice,
        &normalized,
        SynthesisAttempt {
            request_id: &request.job_id,
            owner_kind: "speech_preview",
            owner_id: &voice.revision_id,
            input_revision: &input_revision,
            attempt_id: &request.job_id,
            attempt_epoch: 0,
        },
    )?;
    let mut chunk_seq = 0_u64;
    let chunk_request_id = request.job_id.clone();
    let mut send_chunk = |_: u64, sample_rate: u32, _: u64, pcm: &[u8]| {
        for packet in pcm.chunks(PREVIEW_AUDIO_PACKET_BYTES) {
            chunk_seq = chunk_seq
                .checked_add(1)
                .ok_or("invalid_output: preview chunk sequence overflow")?;
            on_audio_chunk
                .send(SpeechPreviewAudioChunk {
                    request_id: chunk_request_id.clone(),
                    attempt_epoch: 0,
                    seq: chunk_seq,
                    sample_rate,
                    sample_count: u64::try_from(packet.len() / 2)
                        .map_err(|_| "invalid_output: preview chunk size exceeds protocol range")?,
                    pcm16: packet.to_vec(),
                })
                .map_err(|_| "cancelled: preview audio receiver closed")?;
        }
        Ok(())
    };
    let audio = match voice.engine_id.as_str() {
        "kokoro" => {
            let model = preparation::prepare_kokoro_for_job(
                &app,
                &request.job_id,
                "Preparing preset voice",
                lease.token(),
                &mut |resources| send(&on_progress, "preparing", 0, 1, Some(resources)),
            )
            .await
            .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
            send(&on_progress, "synthesizing", 0, 1, None);
            speech::synthesize(
                &app,
                &model,
                kokoro_device_profile(),
                params,
                speech::SynthesisExecution {
                    cancellation: lease.token(),
                    priority: Priority::Interactive,
                    on_chunk: Some(&mut send_chunk),
                },
            )
            .await
        }
        "voxcpm2" => {
            let profile = crate::ai::local_runtime::device_profiles::voxcpm2()?;
            let model = preparation::prepare_voxcpm2_for_job(
                &app,
                &request.job_id,
                "Preparing reference voice",
                lease.token(),
                &mut |resources| send(&on_progress, "preparing", 0, 1, Some(resources)),
            )
            .await
            .map_err(|error| command_error(error, lease.token().is_cancelled()))?;
            send(&on_progress, "synthesizing", 0, 1, None);
            speech::synthesize_voxcpm2(
                &app,
                &model,
                profile,
                params,
                speech::SynthesisExecution {
                    cancellation: lease.token(),
                    priority: Priority::Interactive,
                    on_chunk: Some(&mut send_chunk),
                },
            )
            .await
        }
        _ => return Err("Voice preview engine is unsupported".into()),
    }
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
) -> Result<RunObservation, AppError> {
    let runtime = crate::local_resources::tools::resolve_python_runtime(&app)?
        .ok_or("runtime_missing: managed Python runtime is unavailable")?;
    Ok(run_start::start(db::pool(&app).await?, request, &runtime.digest).await?)
}

#[tauri::command]
pub async fn cancel_speech_generation(job_id: String) -> Result<bool, AppError> {
    uuid::Uuid::parse_str(&job_id).map_err(|_| "Speech job identity is invalid")?;
    Ok(cancellation::cancel(&job_id)?)
}

#[tauri::command]
pub fn discard_speech_preview(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    Ok(publication::discard_preview(&app, &path)?)
}

pub(super) fn parse_config(value: &str) -> Result<VoiceSynthesisConfig, String> {
    parse_synthesis_config(&serde_json::from_str(value).map_err(db::json_error)?)
}

pub(super) fn command_error(error: String, cancelled: bool) -> AppError {
    if cancelled {
        AppError::cancelled("Speech generation cancelled")
    } else {
        error.into()
    }
}

pub(super) fn kokoro_device_profile() -> RuntimeDeviceProfile {
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(4) as u8)
        .unwrap_or(1);
    RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: format!("cpu-int8-threads{threads}-v1"),
        cpu_threads: threads,
    }
}

pub(super) fn digest_text(value: &str) -> String {
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
