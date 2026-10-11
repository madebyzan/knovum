use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use crate::{
    ai::local_runtime::compute_budget::ComputeBudget,
    ai::{
        contracts::{
            runtime::RuntimeDeviceProfile,
            speech_wire::{
                AsrProfile, AudioFileReference, InferenceRequestIdentity, LanguagePolicy,
                SpeechTranscribeParams,
            },
        },
        local_runtime::{admission::Priority, speech},
    },
    local_resources::preparation,
};

use super::recorder::RecordedAudio;
use crate::dictation::contracts::DictationFailure;

pub(crate) struct TranscriptionOutcome {
    pub(crate) text: String,
    pub(crate) language: String,
}

pub(crate) async fn transcribe<R: Runtime>(
    app: &AppHandle<R>,
    session_id: &str,
    wav: PathBuf,
    recording: RecordedAudio,
    cancellation: &CancellationToken,
    progress: &mut (impl FnMut(preparation::contracts::ResourcePreparationProgress) + Send),
) -> Result<TranscriptionOutcome, DictationFailure> {
    let model = crate::local_resources::model_selection::transcription(app)
        .map_err(|error| DictationFailure::classified("local_resource_unavailable", error))?;
    let resources = preparation::prepare_models_for_job(
        app,
        preparation::contracts::ResourcePreparationTarget::Model { model },
        &format!("dictation-resource:{session_id}"),
        "Preparing dictation resources",
        cancellation,
        progress,
    )
    .await
    .map_err(|error| {
        if cancellation.is_cancelled() || error == preparation::RESOURCE_PREPARATION_CANCELLED {
            DictationFailure::Cancelled
        } else {
            DictationFailure::classified("local_resource_unavailable", error)
        }
    })?;
    let audio_ref =
        crate::foundation::blocking::run(move || audio_reference(&wav, recording)).await?;
    let pcm_duration_ms = audio_ref.byte_length.saturating_mul(1_000)
        / (u64::from(audio_ref.sample_rate) * u64::from(audio_ref.channels) * 2);
    let allowance = speech::transcription_budget_ms(model, pcm_duration_ms)
        .map_err(|error| DictationFailure::classified("transcription_failed", error))?;
    let (attempt_epoch, deadline_ms) = deadline(allowance)?;
    let profile = speech::transcription_execution_profile(app, model, cancellation, deadline_ms)
        .await
        .map_err(|error| transcription_failure(cancellation, error))?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let identity = InferenceRequestIdentity {
        request_id: request_id.clone(),
        owner_kind: "desktop_dictation".into(),
        owner_id: session_id.into(),
        attempt_id: request_id,
        attempt_epoch,
        input_revision: format!("sha256:{}", audio_ref.sha256),
        model_spec_id: profile.id.into(),
        deadline_ms,
    };
    let artifact = speech::transcribe(
        app,
        &resources,
        RuntimeDeviceProfile {
            device: profile.device.into(),
            execution_profile_id: profile.id.into(),
            cpu_threads: ComputeBudget::detect()
                .map_err(|error| DictationFailure::classified("transcription_failed", error))?
                .model_threads,
        },
        SpeechTranscribeParams {
            identity,
            audio_ref,
            profile: AsrProfile {
                engine: profile.engine,
                language_policy: LanguagePolicy::Detect,
                language: None,
                device: profile.device.into(),
                compute_type: profile.compute_type.into(),
                beam_size: 5,
                vad_enabled: true,
            },
        },
        cancellation,
        Priority::Interactive,
        |_| {},
    )
    .await
    .map_err(|error| transcription_failure(cancellation, error))?;
    if artifact.language.trim().is_empty() || artifact.language.len() > 64 {
        return Err(DictationFailure::classified(
            "invalid_output",
            "Dictation language is invalid",
        ));
    }
    let mut previous_start = 0;
    for (index, segment) in artifact.segments.iter().enumerate() {
        if segment.text.trim().is_empty()
            || segment.end_ms <= segment.start_ms
            || segment.end_ms > 120_000
            || (index > 0 && segment.start_ms < previous_start)
        {
            return Err(DictationFailure::classified(
                "invalid_output",
                "Dictation segment is invalid",
            ));
        }
        previous_start = segment.start_ms;
    }
    Ok(TranscriptionOutcome {
        text: join_segments(artifact.segments.into_iter().map(|segment| segment.text)),
        language: artifact.language,
    })
}

fn audio_reference(
    path: &Path,
    recording: RecordedAudio,
) -> Result<AudioFileReference, DictationFailure> {
    let byte_length = std::fs::metadata(path)
        .map_err(|error| DictationFailure::classified("audio_capture_failed", error.to_string()))?
        .len();
    let mut file = File::open(path)
        .map_err(|error| DictationFailure::classified("audio_capture_failed", error.to_string()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            DictationFailure::classified("audio_capture_failed", error.to_string())
        })?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(AudioFileReference {
        path: path
            .to_str()
            .ok_or_else(|| {
                DictationFailure::classified(
                    "audio_capture_failed",
                    "Recorded audio path is not valid Unicode",
                )
            })?
            .into(),
        media_type: "audio/wav".into(),
        byte_length,
        sha256: hex::encode(digest.finalize()),
        sample_rate: recording.sample_rate,
        channels: recording.channels,
        sample_format: "pcm_s16le".into(),
    })
}

fn deadline(allowance_ms: u64) -> Result<(u64, u64), DictationFailure> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| DictationFailure::classified("dictation_failed", error.to_string()))?;
    let now_ms = u64::try_from(now.as_millis())
        .map_err(|error| DictationFailure::classified("dictation_failed", error.to_string()))?;
    let deadline_ms = now_ms.checked_add(allowance_ms).ok_or_else(|| {
        DictationFailure::classified(
            "dictation_failed",
            "ASR deadline exceeds the supported range",
        )
    })?;
    Ok((now_ms, deadline_ms))
}

fn transcription_failure(cancellation: &CancellationToken, message: String) -> DictationFailure {
    if cancellation.is_cancelled() {
        DictationFailure::Cancelled
    } else {
        DictationFailure::classified("transcription_failed", message)
    }
}

fn join_segments(segments: impl Iterator<Item = String>) -> String {
    let mut output = String::new();
    for segment in segments {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        if output
            .chars()
            .next_back()
            .zip(segment.chars().next())
            .is_some_and(|(left, right)| {
                left.is_ascii_alphanumeric() && right.is_ascii_alphanumeric()
            })
        {
            output.push(' ');
        }
        output.push_str(segment);
    }
    output
}
