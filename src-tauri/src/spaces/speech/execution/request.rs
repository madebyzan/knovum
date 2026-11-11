use super::parse_config;
use crate::ai::contracts::speech_wire::{
    AudioFileReference, InferenceRequestIdentity, SpeechSynthesisEngine, SpeechSynthesizeParams,
    SynthesisProfile, VoiceRevisionReference, VoiceSourceKind,
    DESIGNED_VOICE_REFERENCE_MIN_DURATION_MS, VOICE_REFERENCE_MIN_DURATION_MS,
};
use crate::spaces::{
    assets,
    speech::{contracts::VoiceExecution, publication},
};
use std::path::Path;

/// Synthesising one segment is bounded work once the voice model is loaded; the load itself is
/// bounded by the worker's startup budget. Generous 10-minute budget accommodates complex models
/// on CPU backends without premature timeouts.
const SYNTHESIS_BUDGET_MS: u64 = 600_000;

#[derive(Clone, Copy)]
pub(in crate::spaces::speech) struct SynthesisAttempt<'a> {
    pub request_id: &'a str,
    pub owner_kind: &'a str,
    pub owner_id: &'a str,
    pub input_revision: &'a str,
    pub attempt_id: &'a str,
    pub attempt_epoch: u64,
}

/// How one rendering of a line is generated: the seed, and for an engine that controls duration,
/// the share of its natural length it is generated at.
#[derive(Clone, Copy)]
pub(in crate::spaces::speech) struct Take {
    pub seed: u32,
    pub duration_scale: Option<f32>,
}

/// `take` is explicit because duration-fitted lines derive further takes from the voice
/// revision's frozen seed; every other caller passes the frozen seed itself at natural length.
pub(in crate::spaces::speech) fn synthesis_params(
    app: &tauri::AppHandle,
    voice: &VoiceExecution,
    text: &str,
    delivery_instruction: Option<&str>,
    take: Take,
    attempt: SynthesisAttempt<'_>,
) -> Result<SpeechSynthesizeParams, String> {
    let config = parse_config(&voice.synthesis_config)?;
    let style = super::super::delivery_instruction::validate(
        delivery_instruction,
        super::super::delivery_instruction::supports(
            &voice.engine_id,
            &voice.source_kind,
            voice.reference_text.as_deref(),
        ),
    )?;
    let engine = SpeechSynthesisEngine::from_id(&voice.engine_id)
        .ok_or("Voice revision engine is unsupported")?;
    let reference = if voice.source_kind == "preset" {
        None
    } else {
        Some(reference_audio(app, voice)?)
    };
    let voice_revision = frozen_voice_revision(voice, reference)?;
    Ok(SpeechSynthesizeParams {
        identity: InferenceRequestIdentity {
            request_id: attempt.request_id.to_owned(),
            owner_kind: attempt.owner_kind.to_owned(),
            owner_id: attempt.owner_id.to_owned(),
            attempt_id: attempt.attempt_id.to_owned(),
            attempt_epoch: attempt.attempt_epoch,
            input_revision: attempt.input_revision.to_owned(),
            model_spec_id: voice.model_spec_id.clone(),
            budget_ms: SYNTHESIS_BUDGET_MS,
        },
        text: text.to_owned(),
        voice_revision,
        style: style.map(str::to_owned),
        seed: u64::from(take.seed),
        duration_scale: take.duration_scale,
        profile: SynthesisProfile {
            engine,
            language: voice.language.clone(),
            speed: config.speed,
            sample_rate: config.sample_rate,
        },
    })
}

fn frozen_voice_revision(
    voice: &VoiceExecution,
    reference_audio: Option<AudioFileReference>,
) -> Result<VoiceRevisionReference, String> {
    let (source_kind, speaker_id, reference_text) =
        match (voice.engine_id.as_str(), voice.source_kind.as_str()) {
            ("moss_nano", "preset") => (
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
            ),
            ("moss_nano", "reference") => (VoiceSourceKind::Reference, None, None),
            ("voxcpm2", "reference" | "designed") | ("qwen3_tts" | "cosyvoice3", "reference") => (
                VoiceSourceKind::Reference,
                None,
                voice.reference_text.clone(),
            ),
            // IndexTTS2 conditions on the recording alone; a transcript has nowhere to go.
            ("index_tts2", "reference") => (VoiceSourceKind::Reference, None, None),
            _ => return Err("Voice revision engine and source are incompatible".into()),
        };
    Ok(VoiceRevisionReference {
        revision_id: voice.revision_id.clone(),
        source_kind,
        speaker_id,
        reference_audio,
        reference_text,
        // Saved designs reuse the selected immutable recording, rather than designing a new voice.
        // A reference recording keeps its style guidance only when it is cloned from the recording
        // alone; with its exact transcript it is cloned by continuation, as spoken.
        design_prompt: if voice.source_kind == "reference" && voice.reference_text.is_none() {
            voice.design_description.clone()
        } else {
            None
        },
    })
}

#[cfg(test)]
#[path = "tests/request.rs"]
mod tests;

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
    let engine = SpeechSynthesisEngine::from_id(&voice.engine_id)
        .ok_or("Voice revision engine is unsupported")?;
    // A designed voice's recording is VoxCPM2's own output, which may be shorter than a recording a
    // person supplies.
    let min_ms = if voice.source_kind == "designed" {
        DESIGNED_VOICE_REFERENCE_MIN_DURATION_MS
    } else {
        VOICE_REFERENCE_MIN_DURATION_MS
    };
    let max_ms = engine.max_reference_duration_ms();
    let reference =
        crate::spaces::speech::voice_assets::reference_metadata_with_bounds(&path, min_ms, max_ms)?;
    if reference.sha256 != sha256 {
        return Err("Reference voice audio changed while it was inspected".into());
    }
    Ok(reference)
}
