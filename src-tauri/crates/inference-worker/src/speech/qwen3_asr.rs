//! Qwen3-ASR transcription on audio.cpp.
//!
//! The Silero VAD selects speech regions on the original waveform, exactly as the Whisper path
//! does, and each region is transcribed as one segment. audio.cpp splits a long region into its
//! own bounded chunks and merges their text, so memory and the generation budget stay flat
//! regardless of source length. Word timing is left to the separate forced-alignment stage: a
//! region carries its text and source span, with no word spans.

use super::{
    artifacts, audio,
    audiocpp::{AudioView, Session, SessionSpec},
    ffi::{
        self,
        whisper::{GgmlApi, WhisperApi},
    },
};
use crate::{
    artifact, cancellation::Cancellation, initialization::VerifiedRuntime, protocol::Output,
};
use knovum_inference_contracts::{
    execution_profile::qwen3_asr,
    runtime::RuntimeInitializeParams,
    speech_wire::{
        AsrEngine, LanguagePolicy, SpeechProgressEvent, SpeechProgressEventType,
        SpeechTranscribeParams, SpeechTranscriptArtifact, SpeechTranscriptSegment,
        SpeechTranscription,
    },
};
use std::{ffi::CString, ops::Range, path::PathBuf};

/// The languages Qwen3-ASR recognizes, as ISO code beside the label the model reads and emits.
const LANGUAGES: [(&str, &str); 30] = [
    ("zh", "Chinese"),
    ("en", "English"),
    ("yue", "Cantonese"),
    ("ar", "Arabic"),
    ("de", "German"),
    ("fr", "French"),
    ("es", "Spanish"),
    ("pt", "Portuguese"),
    ("id", "Indonesian"),
    ("it", "Italian"),
    ("ko", "Korean"),
    ("ru", "Russian"),
    ("th", "Thai"),
    ("vi", "Vietnamese"),
    ("ja", "Japanese"),
    ("tr", "Turkish"),
    ("hi", "Hindi"),
    ("ms", "Malay"),
    ("nl", "Dutch"),
    ("sv", "Swedish"),
    ("da", "Danish"),
    ("fi", "Finnish"),
    ("pl", "Polish"),
    ("cs", "Czech"),
    ("fil", "Filipino"),
    ("fa", "Persian"),
    ("el", "Greek"),
    ("hu", "Hungarian"),
    ("mk", "Macedonian"),
    ("ro", "Romanian"),
];

fn language_label(code: &str) -> Option<&'static str> {
    LANGUAGES
        .into_iter()
        .find(|(iso, _)| *iso == code)
        .map(|(_, label)| label)
}

fn language_code(label: &str) -> Option<&'static str> {
    LANGUAGES
        .into_iter()
        .find(|(_, name)| name.eq_ignore_ascii_case(label))
        .map(|(iso, _)| iso)
}

pub(crate) struct Qwen3Asr {
    session: Session,
    vad_api: WhisperApi,
    _ggml: GgmlApi,
    vad: CString,
    device: String,
    compute_type: &'static str,
    threads: u8,
}

/// Admits a Qwen3-ASR profile and returns the snapshot it names. Pure in its initialization
/// message, so an obsolete profile is rejected before a library or a weight is read.
pub(super) fn admit(params: &RuntimeInitializeParams) -> Result<&str, String> {
    if qwen3_asr(&params.capability_spec_id, &params.device_profile.device).as_deref()
        != Some(params.device_profile.execution_profile_id.as_str())
        || !(1..=8).contains(&params.device_profile.cpu_threads)
    {
        return Err("unsupported_device: Qwen3-ASR profile is unavailable".into());
    }
    params.model.snapshot()
}

impl Qwen3Asr {
    pub(crate) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        let directory = PathBuf::from(admit(params)?);
        let model = directory.join("model.gguf");
        let vad_model = directory.join("vad.bin");
        artifact::bounded_weight(&vad_model, 16 * 1024 * 1024, "Silero VAD")?;
        let vad = ffi::string(
            vad_model
                .to_str()
                .ok_or("model_corrupt: VAD path is invalid")?,
        )?;
        let library = ffi::library(runtime, "whisper")?;
        let ggml_library = library.with_file_name(if cfg!(windows) {
            "ggml.dll"
        } else {
            "libggml.0.dylib"
        });
        let ggml = unsafe { GgmlApi::from_library(ffi::load(&ggml_library)?) }
            .map_err(|error| format!("runtime_mismatch: {error}"))?;
        let vad_api = unsafe { WhisperApi::from_library(ffi::load(&library)?) }
            .map_err(|error| format!("runtime_mismatch: {error}"))?;
        let session = Session::open(
            runtime,
            SessionSpec {
                model: &model,
                family: c"qwen3_asr",
                task: c"asr",
                device: &params.device_profile.device,
                threads: params.device_profile.cpu_threads,
                failure: "transcription_failed",
            },
        )?;
        Ok(Self {
            session,
            vad_api,
            _ggml: ggml,
            vad,
            device: params.device_profile.device.clone(),
            compute_type: if params.capability_spec_id.contains("-f16-") {
                "f16"
            } else {
                "q8_0"
            },
            threads: params.device_profile.cpu_threads,
        })
    }

    pub(crate) fn transcribe(
        &mut self,
        work: &std::path::Path,
        params: SpeechTranscribeParams,
        cancel: &Cancellation,
        output: &Output,
    ) -> Result<SpeechTranscription, String> {
        if !matches!(params.profile.engine, AsrEngine::Qwen3Asr)
            || params.profile.device != self.device
            || params.profile.compute_type != self.compute_type
        {
            return Err("runtime_mismatch: transcription profile differs".into());
        }
        let audio = audio::read(&params.audio_ref, 16_000, 1, 16_000 * 7200, cancel)?;
        let duration_ms = audio.frames() as u64 / 16;
        let regions = if params.profile.vad_enabled {
            super::whisper_vad::regions(
                &self.vad_api,
                &self.vad,
                &audio.samples,
                self.threads,
                cancel,
            )?
        } else {
            std::iter::once(0..duration_ms).collect()
        };
        let fixed_label = match &params.profile.language_policy {
            LanguagePolicy::Detect => None,
            LanguagePolicy::Fixed => Some(
                language_label(
                    params
                        .profile
                        .language
                        .as_deref()
                        .ok_or("invalid_input: fixed language is missing")?,
                )
                .ok_or("unsupported_language: Qwen3-ASR language is unsupported")?,
            ),
        };
        let (language, segments) =
            self.decode(&audio, &regions, fixed_label, cancel, output, &params)?;
        let speech_duration_ms = regions.iter().map(|region| region.end - region.start).sum();
        let transcript = SpeechTranscriptArtifact {
            language,
            duration_ms,
            speech_duration_ms,
            segments,
        };
        let reference = artifacts::json(work, &params.identity.request_id, &transcript)?;
        Ok(SpeechTranscription {
            request_id: params.identity.request_id,
            attempt_epoch: params.identity.attempt_epoch,
            model_spec_id: params.identity.model_spec_id,
            transcript_ref: reference,
        })
    }

    /// Transcribes each speech region in order. A detected language holds for the rest of the
    /// request, because the model recognizes one language per call; an empty first region leaves
    /// detection open until a region attributes a language.
    fn decode(
        &self,
        audio: &audio::Audio,
        regions: &[Range<u64>],
        fixed_label: Option<&'static str>,
        cancel: &Cancellation,
        output: &Output,
        params: &SpeechTranscribeParams,
    ) -> Result<(String, Vec<SpeechTranscriptSegment>), String> {
        let mut detected: Option<&'static str> = None;
        let mut segments = Vec::new();
        for (index, region) in regions.iter().enumerate() {
            cancel.check()?;
            let label = fixed_label.or(detected);
            let (text, language) = self.region(&audio.samples, region, label)?;
            if detected.is_none() {
                detected = language_code(&language);
            }
            let progress = ((index + 1) * 100 / regions.len()).min(99) as u8;
            let text = text.trim();
            let partial = (!text.is_empty()).then(|| SpeechTranscriptSegment {
                start_ms: region.start,
                end_ms: region.end,
                text: text.to_owned(),
                words: Vec::new(),
            });
            output.notification(
                "speech.progress",
                &SpeechProgressEvent {
                    request_id: params.identity.request_id.clone(),
                    attempt_epoch: params.identity.attempt_epoch,
                    seq: (index + 1) as u64,
                    event_type: if partial.is_some() {
                        SpeechProgressEventType::PartialTranscript
                    } else {
                        SpeechProgressEventType::Progress
                    },
                    stage: "transcribing".into(),
                    progress,
                    language: detected.map(str::to_owned),
                    partial: partial.clone(),
                },
            )?;
            if let Some(segment) = partial {
                segments.push(segment);
            }
        }
        let language = match fixed_label {
            Some(label) => language_code(label).unwrap_or("und").to_owned(),
            None => detected.unwrap_or("und").to_owned(),
        };
        Ok((language, segments))
    }

    /// One region's transcript and language label, on the original 16 kHz mono waveform.
    fn region(
        &self,
        samples: &[f32],
        region: &Range<u64>,
        label: Option<&str>,
    ) -> Result<(String, String), String> {
        let start = region.start as usize * 16;
        let end = region.end as usize * 16;
        let mut request = self.session.request()?;
        request.audio(AudioView {
            samples: &samples[start..end],
            sample_rate: 16_000,
            channels: 1,
        })?;
        request.option(c"audio_chunk_mode", "fixed")?;
        request.option(c"return_timestamps", "false")?;
        if let Some(label) = label {
            request.option(c"language", label)?;
        }
        self.session.run(&request)?.transcript()
    }
}

#[cfg(test)]
#[path = "tests/qwen3_asr.rs"]
mod tests;
