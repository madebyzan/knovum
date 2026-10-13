use super::parse_config;
use crate::ai::contracts::speech_wire::{
    AudioFileReference, InferenceRequestIdentity, SpeechSynthesisEngine, SpeechSynthesizeParams,
    SynthesisProfile, VoiceRevisionReference, VoiceSourceKind,
};
use crate::spaces::{
    assets,
    speech::{contracts::VoiceExecution, publication},
};
use std::path::Path;

/// Synthesising one segment is bounded work once the voice model is loaded; the load itself is
/// bounded by the worker's startup budget.
const SYNTHESIS_BUDGET_MS: u64 = 300_000;

pub(in crate::spaces::speech) struct SynthesisAttempt<'a> {
    pub request_id: &'a str,
    pub owner_kind: &'a str,
    pub owner_id: &'a str,
    pub input_revision: &'a str,
    pub attempt_id: &'a str,
    pub attempt_epoch: u64,
}

pub(in crate::spaces::speech) fn synthesis_params(
    app: &tauri::AppHandle,
    voice: &VoiceExecution,
    text: &str,
    delivery_instruction: Option<&str>,
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
    let engine = match voice.engine_id.as_str() {
        "moss_nano" => SpeechSynthesisEngine::MossNano,
        "voxcpm2" => SpeechSynthesisEngine::VoxCpm2,
        _ => return Err("Voice revision engine is unsupported".into()),
    };
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
        seed: u64::from(config.seed),
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
            ("voxcpm2", "reference" | "designed") => (
                VoiceSourceKind::Reference,
                None,
                voice.reference_text.clone(),
            ),
            _ => return Err("Voice revision engine and source are incompatible".into()),
        };
    Ok(VoiceRevisionReference {
        revision_id: voice.revision_id.clone(),
        source_kind,
        speaker_id,
        reference_audio,
        reference_text,
        // Saved designs reuse the selected immutable recording, rather than designing a new voice.
        design_prompt: None,
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
    let reference = crate::spaces::speech::voice_assets::reference_metadata(&path)?;
    if reference.sha256 != sha256 {
        return Err("Reference voice audio changed while it was inspected".into());
    }
    Ok(reference)
}
