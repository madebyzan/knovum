//! Voice cloning through the audio.cpp families: IndexTTS2, Qwen3-TTS and CosyVoice3.
//!
//! Each clones the speaker of an immutable reference recording. A confirmed transcript of the
//! recording lets Qwen3-TTS and CosyVoice3 continue from it (in-context cloning), which carries
//! the reference's delivery as well as its timbre; without one they clone the timbre alone.
//! IndexTTS2 is the one family that generates a line to a requested length, which is what lets a
//! dub land in its time without time-stretching the voice.

use super::{
    audio,
    audiocpp::{AudioView, Session, SessionSpec},
    stream::Publisher,
};
use crate::{cancellation::Cancellation, initialization::VerifiedRuntime};
use knovum_inference_contracts::{
    execution_profile::voice,
    runtime::RuntimeInitializeParams,
    speech_wire::{
        voice_reference_sample_bounds, SpeechSynthesisEngine, SpeechSynthesizeParams,
        VoiceSourceKind, MIN_DURATION_SCALE, VOICE_REFERENCE_MIN_DURATION_MS,
        VOICE_REFERENCE_SAMPLE_RATE,
    },
};
use std::{ffi::CStr, path::PathBuf};

/// Lines are published in chunks of this many 48 kHz samples (four seconds).
const CHUNK_SAMPLES: usize = 4 * 48_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Family {
    IndexTts2,
    Qwen3Tts,
    CosyVoice3,
}

impl Family {
    pub(super) const fn engine(self) -> SpeechSynthesisEngine {
        match self {
            Self::IndexTts2 => SpeechSynthesisEngine::IndexTts2,
            Self::Qwen3Tts => SpeechSynthesisEngine::Qwen3Tts,
            Self::CosyVoice3 => SpeechSynthesisEngine::CosyVoice3,
        }
    }

    const fn library_name(self) -> &'static CStr {
        match self {
            Self::IndexTts2 => c"index_tts2",
            Self::Qwen3Tts => c"qwen3_tts",
            Self::CosyVoice3 => c"cosyvoice3",
        }
    }
}

/// Admits a cloning profile and returns its weights and family. Pure in its initialization
/// message, so an obsolete profile is rejected before the audio.cpp library is loaded.
pub(super) fn admit(params: &RuntimeInitializeParams) -> Result<(PathBuf, Family), String> {
    let (family, weights) = match params.capability_spec_id.as_str() {
        "index-tts2-q8_0-v1" => (Family::IndexTts2, "index-tts2-q8_0.gguf"),
        "index-tts2-f16-v1" => (Family::IndexTts2, "index-tts2-f16.gguf"),
        "qwen3-tts-1.7b-q8_0-v1" => (Family::Qwen3Tts, "qwen3-tts-12hz-1.7b-base-q8_0_v2.gguf"),
        "qwen3-tts-1.7b-bf16-v1" => (Family::Qwen3Tts, "qwen3-tts-12hz-1.7b-base-bf16.gguf"),
        "qwen3-tts-0.6b-q8_0-v1" => (Family::Qwen3Tts, "qwen3-tts-12hz-0.6b-base-q8_0.gguf"),
        "cosyvoice3-0.5b-q8_0-v1" => (Family::CosyVoice3, "cosyvoice3-q8_0.gguf"),
        _ => return Err("runtime_mismatch: voice cloning model specification differs".into()),
    };
    if voice(&params.capability_spec_id, &params.device_profile.device).as_deref()
        != Some(params.device_profile.execution_profile_id.as_str())
        || !(1..=8).contains(&params.device_profile.cpu_threads)
    {
        return Err("runtime_mismatch: voice cloning profile is unavailable".into());
    }
    Ok((
        PathBuf::from(params.model.snapshot()?).join(weights),
        family,
    ))
}

pub(crate) struct Cloner {
    session: Session,
    family: Family,
}

impl Cloner {
    pub(super) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        let (model, family) = admit(params)?;
        let session = Session::open(
            runtime,
            SessionSpec {
                model: &model,
                family: family.library_name(),
                task: c"clone",
                device: &params.device_profile.device,
                threads: params.device_profile.cpu_threads,
                failure: "synthesis_failed",
            },
        )?;
        Ok(Self { session, family })
    }

    pub(super) fn family(&self) -> Family {
        self.family
    }

    pub(super) fn generate(
        &mut self,
        params: &SpeechSynthesizeParams,
        publisher: &mut Publisher,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if params.duration_scale.is_some() && !self.family.engine().controls_duration() {
            return Err("invalid_input: this engine speaks every line at its own pace".into());
        }
        let reference = self.reference(params, cancel)?;
        let mut request = self.session.request()?;
        request.text(&params.text)?;
        request.voice_audio(AudioView {
            samples: &reference,
            sample_rate: VOICE_REFERENCE_SAMPLE_RATE,
            channels: 1,
        })?;
        request.option(c"seed", &params.seed.to_string())?;
        let transcript = params.voice_revision.reference_text.as_deref();
        let style = style(params);
        match self.family {
            Family::IndexTts2 => {
                if let Some(style) = style {
                    request.option(c"use_emotion_text", "true")?;
                    request.option(c"emotion_text", style)?;
                }
                if let Some(scale) = params.duration_scale {
                    if !(MIN_DURATION_SCALE..=1.0).contains(&scale) {
                        return Err("invalid_input: duration scale is out of range".into());
                    }
                    request.option(c"duration_factor", &scale.to_string())?;
                }
            }
            Family::Qwen3Tts => {
                if style.is_some() {
                    return Err("invalid_input: Qwen3-TTS does not accept delivery styles".into());
                }
                request.option(c"language", qwen_language(&params.profile.language)?)?;
                match transcript {
                    Some(transcript) => request.option(c"reference_text", transcript)?,
                    None => request.option(c"x_vector_only_mode", "true")?,
                }
            }
            Family::CosyVoice3 => match (style, transcript) {
                (Some(style), _) => {
                    request.option(c"template_name", "instruct")?;
                    request.option(c"instruction", style)?;
                }
                (None, Some(transcript)) => {
                    request.option(c"template_name", "zero_shot")?;
                    request.option(c"reference_text", transcript)?;
                }
                (None, None) => request.option(c"template_name", "cross_lingual")?,
            },
        }
        cancel.check()?;
        let output = self.session.run(&request)?;
        let audio = output.audio()?;
        if audio.channels != 1 {
            return Err("invalid_output: voice cloning returned more than one channel".into());
        }
        let samples = audio::resample(audio.samples.to_vec(), audio.sample_rate, 48_000)?;
        drop(output);
        cancel.check()?;
        for chunk in samples.chunks(CHUNK_SAMPLES) {
            publisher.publish(chunk, 48_000)?;
        }
        Ok(())
    }

    /// The reference recording, at 16 kHz mono, within the family's bounds.
    fn reference(
        &self,
        params: &SpeechSynthesizeParams,
        cancel: &Cancellation,
    ) -> Result<Vec<f32>, String> {
        let voice = &params.voice_revision;
        if !matches!(voice.source_kind, VoiceSourceKind::Reference) || voice.speaker_id.is_some() {
            return Err("invalid_input: voice cloning requires a reference recording".into());
        }
        if voice
            .reference_text
            .as_deref()
            .is_some_and(|text| text.trim().is_empty())
        {
            return Err("invalid_input: reference transcript is empty".into());
        }
        let reference = voice
            .reference_audio
            .as_ref()
            .ok_or("invalid_input: immutable reference audio is missing")?;
        let (min_samples, max_samples) = voice_reference_sample_bounds(
            VOICE_REFERENCE_MIN_DURATION_MS,
            self.family.engine().max_reference_duration_ms(),
        );
        let audio = audio::read(
            reference,
            VOICE_REFERENCE_SAMPLE_RATE,
            1,
            max_samples as usize,
            cancel,
        )?;
        if audio.frames() < min_samples as usize {
            return Err("invalid_input: voice reference is shorter than five seconds".into());
        }
        Ok(audio.samples)
    }
}

/// The delivery style of a line: the voice's own guidance and the line's instruction, together.
fn style(params: &SpeechSynthesizeParams) -> Option<&str> {
    params
        .style
        .as_deref()
        .or(params.voice_revision.design_prompt.as_deref())
        .map(str::trim)
        .filter(|style| !style.is_empty())
}

/// The language name Qwen3-TTS's codec is conditioned on, for a BCP 47 tag.
pub(super) fn qwen_language(tag: &str) -> Result<&'static str, String> {
    let primary = tag.split(['-', '_']).next().unwrap_or_default();
    Ok(match primary.to_ascii_lowercase().as_str() {
        "zh" => "chinese",
        "en" => "english",
        "ja" => "japanese",
        "ko" => "korean",
        "de" => "german",
        "fr" => "french",
        "ru" => "russian",
        "pt" => "portuguese",
        "es" => "spanish",
        "it" => "italian",
        _ => return Err(format!("invalid_input: Qwen3-TTS does not speak {tag}")),
    })
}

#[cfg(test)]
#[path = "tests/cloning.rs"]
mod tests;
