use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use tauri::ipc::Channel;

use super::{cancellation, contracts::*, publication, run_start};
use crate::{
    ai::{
        contracts::{
            runtime::RuntimeDeviceProfile,
            speech_wire::{
                AudioFileReference, InferenceRequestIdentity, SpeechSynthesisEngine,
                SpeechSynthesizeParams, SynthesisProfile, VoiceRevisionReference, VoiceSourceKind,
            },
        },
        local_runtime::{admission::Priority, speech},
    },
    app_error::AppError,
    local_resources::preparation,
    spaces::{assets, db, runs::RunObservation},
};

pub(crate) const KOKORO_MODEL_SPEC_ID: &str = "kokoro-int8-multi-lang-v1_0";
// Preview packet sequence is a bounded IPC transport sequence. The runtime validates the
// originating model-chunk or sentence sequence before this presentation-only split.
const PREVIEW_AUDIO_PACKET_BYTES: usize = 128 * 1024;

pub(super) struct SynthesisAttempt<'a> {
    pub request_id: &'a str,
    pub owner_kind: &'a str,
    pub owner_id: &'a str,
    pub input_revision: &'a str,
    pub attempt_id: &'a str,
    pub attempt_epoch: u64,
}

#[tauri::command]
pub async fn preview_voice(
    app: tauri::AppHandle,
    request: PreviewVoiceRequest,
    on_progress: Channel<SpeechGenerationProgress>,
    on_audio_chunk: Channel<SpeechPreviewAudioChunk>,
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
    Ok(cancellation::cancel(&job_id, false)?)
}

#[tauri::command]
pub fn discard_speech_preview(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    Ok(publication::discard_preview(&app, &path)?)
}

pub(super) fn synthesis_params(
    app: &tauri::AppHandle,
    voice: &VoiceExecution,
    text: &str,
    attempt: SynthesisAttempt<'_>,
) -> Result<SpeechSynthesizeParams, String> {
    let config = parse_config(&voice.synthesis_config)?;
    let (engine, source_kind, speaker_id, reference_audio, reference_text) =
        match (voice.engine_id.as_str(), voice.source_kind.as_str()) {
            ("kokoro", "preset") => (
                SpeechSynthesisEngine::Kokoro,
                VoiceSourceKind::Preset,
                Some(
                    voice
                        .preset_voice_id
                        .as_deref()
                        .ok_or("Preset speaker identity is missing")?
                        .parse()
                        .map_err(|_| "Preset speaker identity is invalid")?,
                ),
                None,
                None,
            ),
            ("voxcpm2", "reference" | "designed") => (
                SpeechSynthesisEngine::VoxCpm2,
                VoiceSourceKind::Reference,
                None,
                Some(reference_audio(app, voice)?),
                voice.reference_text.clone(),
            ),
            _ => return Err("Voice revision engine and source are incompatible".into()),
        };
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
            source_kind,
            speaker_id,
            reference_audio,
            reference_text,
            design_prompt: voice.design_description.clone(),
        },
        style: None,
        seed: 0,
        profile: SynthesisProfile {
            engine,
            language: voice.language.clone(),
            speed: config.speed,
            sample_rate: config.sample_rate,
        },
    })
}

fn reference_audio(
    app: &tauri::AppHandle,
    voice: &VoiceExecution,
) -> Result<AudioFileReference, String> {
    let stored = voice
        .reference_asset_id
        .as_deref()
        .ok_or("Reference voice audio is missing")?;
    let path = assets::authorize_spaces_asset(app, Path::new(stored))?;
    let sha256 = publication::hash_file(&path)?;
    let expected = voice
        .reference_source_revision
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .ok_or("Reference voice source revision is invalid")?;
    if sha256 != expected {
        return Err("Reference voice audio differs from its immutable revision".into());
    }
    let reference = super::voice_assets::reference_metadata(&path)?;
    if reference.sha256 != sha256 {
        return Err("Reference voice audio changed while it was inspected".into());
    }
    Ok(reference)
}

pub(super) fn parse_config(value: &str) -> Result<PresetSynthesisConfig, String> {
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

pub(super) async fn load_voice(
    pool: &sqlx::SqlitePool,
    revision_id: &str,
    space_id: &str,
) -> Result<VoiceExecution, String> {
    let voice: VoiceExecution = sqlx::query_as(
        "SELECT r.id AS revision_id,p.source_kind,r.engine_id,r.model_spec_id,r.language,          r.preset_voice_id,r.reference_asset_id,r.reference_source_revision,r.reference_text,          r.design_description,r.synthesis_config FROM spaces_voice_revisions r          JOIN spaces_voice_profiles p ON p.id=r.profile_id WHERE r.id=? AND p.space_id=?",
    )
    .bind(revision_id).bind(space_id).fetch_optional(pool).await.map_err(db::sql_error)?
    .ok_or("Voice revision is unavailable")?;
    match (voice.engine_id.as_str(), voice.source_kind.as_str()) {
        ("kokoro", "preset") if voice.model_spec_id == KOKORO_MODEL_SPEC_ID => {}
        ("voxcpm2", "reference" | "designed")
            if voice.model_spec_id == super::voice_candidates::MODEL_SPEC_ID
                && voice.reference_asset_id.is_some()
                && voice.reference_source_revision.is_some()
                && (voice.source_kind == "reference"
                    || voice
                        .reference_text
                        .as_deref()
                        .is_some_and(|text| !text.trim().is_empty())) => {}
        _ => return Err("Voice revision has an unsupported production configuration".into()),
    }
    let config = parse_config(&voice.synthesis_config)?;
    if voice.engine_id == "voxcpm2" && (config.sample_rate != 48_000 || config.speed != 1.0) {
        return Err("VoxCPM2 voice revision requires its fixed 48 kHz profile".into());
    }
    Ok(voice)
}

pub(super) fn input_digest(
    segment: &SpeechSegmentExecution,
    voice: &VoiceExecution,
    config: &PresetSynthesisConfig,
    runtime_digest: &str,
) -> Result<String, String> {
    let execution_profile_id = match voice.engine_id.as_str() {
        "kokoro" => kokoro_device_profile().execution_profile_id,
        "voxcpm2" => crate::ai::local_runtime::device_profiles::voxcpm2()?.execution_profile_id,
        _ => return Err("Speech voice uses an unsupported engine".into()),
    };
    let value = serde_json::json!({
        "text": segment.normalized_text,
        "voiceRevisionId": voice.revision_id,
        "engineId": voice.engine_id,
        "modelSpecId": voice.model_spec_id,
        "language": segment.language,
        "sourceRevision": voice.reference_source_revision,
        "synthesis": config,
        "executionProfileId": execution_profile_id,
        "seed": 0,
        "engineVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest,
        "preprocessingVersion": "speech-text-v1",
    });
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(db::json_error)?)
    ))
}
