use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::{send_partial_progress, send_progress, MediaTranscriptionProgress};
use crate::{
    ai::{
        contracts::{
            runtime::RuntimeDeviceProfile,
            speech_wire::{
                AsrEngine, AsrProfile, AudioFileReference, InferenceRequestIdentity,
                LanguagePolicy, SpeechTranscribeParams,
            },
        },
        local_runtime::{admission::Priority, speech},
    },
    blocking, file_inspection,
    local_resources::preparation::PreparedModels,
    media::{
        compute_budget::ComputeBudget,
        prepared_audio::PreparedMediaAudio,
        transcript::{from_speech, MediaTranscript},
    },
};

pub struct TranscriptionPipeline<'a> {
    pub app: &'a tauri::AppHandle,
    pub job_id: &'a str,
    pub source_revision: &'a str,
    pub progress: &'a Channel<MediaTranscriptionProgress>,
    pub cancellation: &'a CancellationToken,
    pub resources: &'a PreparedModels,
}

impl TranscriptionPipeline<'_> {
    pub async fn run(&self, audio: &PreparedMediaAudio) -> Result<MediaTranscript, String> {
        let (engine, device, compute_type, profile_id) = platform_profile()?;
        let vad_enabled = true;
        let budget = ComputeBudget::detect()?;
        let threads = budget.model_threads;
        let wav = audio.wav.clone();
        let audio_ref = blocking::run(move || audio_reference(&wav)).await?;
        let (attempt_epoch, deadline_ms) = deadline(audio.analysis.duration_ms)?;
        let request_id = uuid::Uuid::new_v4().to_string();
        let params = SpeechTranscribeParams {
            identity: InferenceRequestIdentity {
                request_id: request_id.clone(),
                owner_kind: "media_transcription".into(),
                owner_id: self.job_id.into(),
                attempt_id: request_id,
                attempt_epoch,
                input_revision: self.source_revision.into(),
                model_spec_id: profile_id.into(),
                deadline_ms,
            },
            audio_ref,
            profile: AsrProfile {
                engine,
                language_policy: LanguagePolicy::Detect,
                language: None,
                device: device.into(),
                compute_type: compute_type.into(),
                beam_size: 5,
                vad_enabled,
            },
        };
        send_progress(self.progress, "transcribing", 18);
        let result = speech::transcribe(
            self.app,
            self.resources,
            RuntimeDeviceProfile {
                device: device.into(),
                execution_profile_id: profile_id.into(),
                cpu_threads: threads,
            },
            params,
            self.cancellation,
            Priority::Background,
            |event| {
                let progress = 18 + (u16::from(event.progress) * 76 / 100) as u8;
                if let Some(segment) = event.partial {
                    send_partial_progress(
                        self.progress,
                        progress,
                        event.language.as_deref().unwrap_or("unknown"),
                        vec![crate::media::transcript::TranscriptSegment {
                            start_ms: segment.start_ms,
                            end_ms: segment.end_ms,
                            text: segment.text,
                        }],
                    );
                } else {
                    send_progress(self.progress, "transcribing", progress);
                }
            },
        )
        .await?;
        let transcript = from_speech(result, audio.analysis.duration_ms)?;
        send_progress(self.progress, "parsing", 96);
        Ok(transcript)
    }
}

fn audio_reference(wav: &Path) -> Result<AudioFileReference, String> {
    let length = std::fs::metadata(wav)
        .map_err(|error| error.to_string())?
        .len();
    let revision = file_inspection::content_revision(wav)?;
    let sha256 = revision
        .strip_prefix("sha256:")
        .ok_or("Prepared audio digest is invalid")?;
    let path = wav
        .to_str()
        .ok_or("Prepared audio path is not valid Unicode")?;
    Ok(AudioFileReference {
        path: path.into(),
        media_type: "audio/wav".into(),
        byte_length: length,
        sha256: sha256.into(),
        sample_rate: 16_000,
        channels: 1,
        sample_format: "pcm_s16le".into(),
    })
}

fn deadline(duration_ms: u64) -> Result<(u64, u64), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    let now_ms = u64::try_from(now.as_millis()).map_err(|error| error.to_string())?;
    let allowance = duration_ms
        .saturating_mul(4)
        .clamp(120_000, 6 * 60 * 60 * 1_000);
    let deadline_ms = now_ms
        .checked_add(allowance)
        .ok_or("ASR deadline exceeds the supported range")?;
    Ok((now_ms, deadline_ms))
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(super) fn platform_profile(
) -> Result<(AsrEngine, &'static str, &'static str, &'static str), String> {
    Ok((
        AsrEngine::MlxWhisper,
        "mps",
        "float16",
        "mlx-whisper-small-mps-v1",
    ))
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub(super) fn platform_profile(
) -> Result<(AsrEngine, &'static str, &'static str, &'static str), String> {
    Ok((
        AsrEngine::FasterWhisper,
        "cpu",
        "int8",
        "faster-whisper-small-cpu-int8-v1",
    ))
}

#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
pub(super) fn platform_profile(
) -> Result<(AsrEngine, &'static str, &'static str, &'static str), String> {
    Err("unsupported_device: native ASR runtime has not been qualified for this platform".into())
}
