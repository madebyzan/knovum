use super::{
    audio,
    audiocpp::{AudioView, Session, SessionSpec},
};
use crate::{cancellation::Cancellation, initialization::VerifiedRuntime};
use knovum_inference_contracts::{
    execution_profile::alignment,
    runtime::RuntimeInitializeParams,
    speech_wire::{AlignedSpeechUnit, AlignmentUnitKind, SpeechAlignParams, SpeechAlignment},
};
use std::path::PathBuf;

pub(crate) struct Aligner {
    session: Session,
    device: String,
}

/// Admits an alignment profile and returns the snapshot it names. Pure in its initialization
/// message, so an obsolete profile is rejected before the audio.cpp library is loaded.
pub(super) fn admit(params: &RuntimeInitializeParams) -> Result<&str, String> {
    if alignment(&params.capability_spec_id, &params.device_profile.device).as_deref()
        != Some(params.device_profile.execution_profile_id.as_str())
        || !(1..=8).contains(&params.device_profile.cpu_threads)
    {
        return Err("unsupported_device: alignment profile is unavailable".into());
    }
    params.model.snapshot()
}

impl Aligner {
    pub(crate) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        let directory = PathBuf::from(admit(params)?);
        let model = directory.join(
            if params.capability_spec_id == "qwen3-forced-aligner-0.6b-f16-bundled-v2" {
                "qwen3-forced-aligner-0.6b-f16.gguf"
            } else {
                "qwen3-forced-aligner-0.6b-q8_0.gguf"
            },
        );
        let session = Session::open(
            runtime,
            SessionSpec {
                model: &model,
                family: c"qwen3_forced_aligner",
                task: c"align",
                device: &params.device_profile.device,
                threads: params.device_profile.cpu_threads,
                failure: "alignment_failed",
            },
        )?;
        Ok(Self {
            session,
            device: params.device_profile.device.clone(),
        })
    }

    pub(crate) fn align(
        &mut self,
        params: SpeechAlignParams,
        cancel: &Cancellation,
    ) -> Result<SpeechAlignment, String> {
        if params.profile.device != self.device || !(1..=8).contains(&params.text_segments.len()) {
            return Err("runtime_mismatch: alignment request differs from its session".into());
        }
        let language = match params.profile.language.as_str() {
            "zh" => "Chinese",
            "en" => "English",
            "yue" => "Cantonese",
            "fr" => "French",
            "de" => "German",
            "it" => "Italian",
            "pt" => "Portuguese",
            "ru" => "Russian",
            "es" => "Spanish",
            _ => return Err("unsupported_language: alignment language is unsupported".into()),
        };
        let audio = audio::read(&params.audio_ref, 16_000, 1, 16_000 * 7200, cancel)?;
        for segment in &params.text_segments {
            if segment.text.trim().is_empty()
                || segment.text.len() > 65_536
                || segment.end_ms <= segment.start_ms
                || segment.end_ms - segment.start_ms > 300_000
                || segment.end_ms > audio.frames() as u64 / 16
            {
                return Err("invalid_input: alignment segment exceeds its source timeline".into());
            }
        }
        cancel.check()?;
        let mut units = Vec::new();
        for segment in params.text_segments {
            let start = segment.start_ms as usize * 16;
            let end = segment.end_ms as usize * 16;
            let words = self.words(&audio.samples[start..end], &segment.text, language)?;
            cancel.check()?;
            let kind = if matches!(language, "Chinese" | "Cantonese") {
                AlignmentUnitKind::Character
            } else {
                AlignmentUnitKind::Word
            };
            if words.is_empty() {
                units.push(AlignedSpeechUnit {
                    segment_id: segment.segment_id,
                    text: segment.text,
                    unit_kind: kind,
                    start_ms: None,
                    end_ms: None,
                    score: None,
                    unaligned_reason: Some("no_alignment".into()),
                });
                continue;
            }
            for (word, start, end) in words {
                units.push(AlignedSpeechUnit {
                    segment_id: segment.segment_id.clone(),
                    text: word,
                    unit_kind: kind.clone(),
                    start_ms: Some(segment.start_ms + start),
                    end_ms: Some(segment.start_ms + end),
                    score: None,
                    unaligned_reason: None,
                });
            }
        }
        let coverage =
            units.iter().filter(|unit| unit.start_ms.is_some()).count() as f32 / units.len() as f32;
        Ok(SpeechAlignment {
            request_id: params.identity.request_id,
            attempt_epoch: params.identity.attempt_epoch,
            model_spec_id: params.identity.model_spec_id,
            units,
            coverage,
        })
    }

    /// The words of `text` in `samples` (16 kHz mono), as milliseconds into those samples.
    fn words(
        &self,
        samples: &[f32],
        text: &str,
        language: &str,
    ) -> Result<Vec<(String, u64, u64)>, String> {
        let mut request = self.session.request()?;
        request.text(text)?;
        request.text_language(language)?;
        request.audio(AudioView {
            samples,
            sample_rate: 16_000,
            channels: 1,
        })?;
        let words = self.session.run(&request)?.words()?;
        if words.len() > text.chars().count() {
            return Err("invalid_output: alignment exceeds its transcript length".into());
        }
        words
            .into_iter()
            .map(|word| {
                if word.end as usize > samples.len() || word.text.is_empty() {
                    return Err("invalid_output: alignment returned an invalid word".into());
                }
                Ok((
                    word.text,
                    (word.start * 1000).div_ceil(16_000),
                    (word.end * 1000).div_ceil(16_000),
                ))
            })
            .collect()
    }
}
