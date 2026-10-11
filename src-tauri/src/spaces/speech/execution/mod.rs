pub(super) use voice::{input_digest, load_voice};
mod voice;
pub(super) use request::{synthesis_params, SynthesisAttempt};
mod request;
mod resources;
pub(in crate::spaces::speech) use resources::{prepare_synthesis, synthesis_device_profile};

use sha2::{Digest, Sha256};
use tauri::ipc::Channel;

use super::{
    cancellation,
    contracts::{
        parse_synthesis_config, GenerateSpeechProjectRequest, PreviewVoiceRequest,
        SpeechAudioOutput, SpeechGenerationProgress, SpeechPreviewAudioChunk, VoiceExecution,
        VoicePreviewSource, VoiceSynthesisConfig,
    },
    publication, run_start,
};
use crate::{
    ai::{
        contracts::runtime::RuntimeDeviceProfile,
        local_runtime::{admission::Priority, speech},
    },
    foundation::app_error::AppError,
    local_resources::preparation,
    spaces::{db, runs::RunObservation},
};

pub(crate) const MOSS_NANO_MODEL_SPEC_ID: &str = "moss-tts-nano-100m-onnx-native-v2";
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
    let (cancellation, _lease) = cancellation::acquire(&request.job_id)?;
    let pool = db::pool(&app).await?;
    let voice = match &request.source {
        VoicePreviewSource::Revision { revision_id } => load_voice(pool, revision_id).await?,
        VoicePreviewSource::Preset {
            speaker_id,
            language,
        } => {
            super::preset_voices::require(speaker_id, language)?;
            VoiceExecution {
                revision_id: format!("moss_nano-preset:{speaker_id}"),
                source_kind: "preset".into(),
                engine_id: "moss_nano".into(),
                model_spec_id: MOSS_NANO_MODEL_SPEC_ID.into(),
                language: language.clone(),
                preset_voice_id: Some(speaker_id.clone()),
                reference_asset_id: None,
                reference_source_revision: None,
                reference_text: None,
                design_description: None,
                synthesis_config: serde_json::json!({ "speed": 1, "sampleRate": 48000, "seed": 0 })
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
    let delivery = super::delivery_instruction::validate(
        request.delivery_instruction.as_deref(),
        super::delivery_instruction::supports(
            &voice.engine_id,
            &voice.source_kind,
            voice.reference_text.as_deref(),
        ),
    )?;
    send(&on_progress, "preparing", 0, 1, None);
    let input_revision = hex::encode(Sha256::digest(
        serde_json::to_vec(&(normalized.as_str(), delivery)).map_err(db::json_error)?,
    ));
    let model = prepare_synthesis(
        &app,
        &voice.engine_id,
        &voice.model_spec_id,
        &request.job_id,
        "Preparing voice preview",
        &cancellation,
        &mut |resources| send(&on_progress, "preparing", 0, 1, Some(resources)),
    )
    .await
    .map_err(|error| command_error(error, cancellation.is_cancelled()))?;
    let params = synthesis_params(
        &app,
        &voice,
        &normalized,
        delivery,
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
    send(&on_progress, "synthesizing", 0, 1, None);
    let audio = model
        .synthesize(
            &app,
            params,
            speech::SynthesisExecution {
                cancellation: &cancellation,
                priority: Priority::Interactive,
                on_chunk: Some(&mut send_chunk),
            },
        )
        .await
        .map_err(|error| command_error(error, cancellation.is_cancelled()))?;
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled("Speech preview cancelled"));
    }
    if audio.sample_rate != parse_config(&voice.synthesis_config)?.sample_rate {
        return Err(
            "invalid_output: synthesized sample rate differs from the voice revision".into(),
        );
    }
    let output = publication::write_preview(&app, &request.job_id, audio)?;
    if cancellation.is_cancelled() {
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
    let pool = db::pool(&app).await?;
    let (cancellation, _lease) = cancellation::acquire(&request.request_id)?;
    let digests = super::runtime_identity::resolve(
        &app,
        pool,
        &request.project_id,
        &request.request_id,
        &cancellation,
    )
    .await
    .map_err(|error| command_error(error, cancellation.is_cancelled()))?;
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled("Speech preparation cancelled"));
    }
    let run = run_start::start(pool, request, &digests).await?;
    if cancellation.is_cancelled() {
        cancellation::cancel_project_run(pool, &run.id).await?;
    }
    Ok(run)
}

#[tauri::command]
pub async fn cancel_speech_generation(
    app: tauri::AppHandle,
    job_id: String,
) -> Result<bool, AppError> {
    uuid::Uuid::parse_str(&job_id).map_err(|_| "Speech job identity is invalid")?;
    let preparing = cancellation::cancel(&job_id)?;
    let pool = db::pool(&app).await?;
    let running = cancellation::cancel_project_run(pool, &job_id).await?;
    Ok(preparing || running)
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

pub(super) fn moss_nano_device_profile() -> RuntimeDeviceProfile {
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(4) as u8)
        .unwrap_or(1);
    RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: format!("moss-nano-cpu-f32-threads{threads}-native-v2"),
        cpu_threads: threads,
    }
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
