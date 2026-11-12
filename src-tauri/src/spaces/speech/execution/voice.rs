use super::{parse_config, MOSS_NANO_MODEL_SPEC_ID};
use crate::spaces::{
    db,
    speech::contracts::{SpeechSegmentExecution, VoiceExecution, VoiceSynthesisConfig},
};
use sha2::{Digest, Sha256};

/// Every column production reads from a voice revision.
pub(in crate::spaces::speech) const VOICE_QUERY: &str = "SELECT r.id AS revision_id,p.source_kind,\
     r.engine_id,r.model_spec_id,r.language,r.preset_voice_id,r.reference_asset_id,\
     r.reference_source_revision,r.reference_text,r.design_description,r.synthesis_config \
     FROM spaces_voice_revisions r JOIN spaces_voice_profiles p ON p.id=r.profile_id WHERE r.id=?";

pub(in crate::spaces::speech) async fn load_voice(
    pool: &sqlx::SqlitePool,
    revision_id: &str,
) -> Result<VoiceExecution, String> {
    let voice: VoiceExecution = sqlx::query_as(VOICE_QUERY)
        .bind(revision_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("Voice revision is unavailable")?;
    admit(&voice)?;
    Ok(voice)
}

/// Whether production can speak with a stored voice revision: its engine, model, source and
/// conditioning must form a configuration some engine renders. The one rule every caller uses.
pub(in crate::spaces::speech) fn admit(voice: &VoiceExecution) -> Result<(), String> {
    // The engine the stored model speaks with must be the engine the revision names.
    let model_engine =
        crate::local_resources::model_catalog::LocalModelId::from_voice_spec(&voice.model_spec_id)
            .ok()
            .and_then(|model| model.voice_engine())
            .map(|engine| engine.id());
    let recorded = voice.reference_asset_id.is_some() && voice.reference_source_revision.is_some();
    match (voice.engine_id.as_str(), voice.source_kind.as_str()) {
        ("moss_nano", "preset") if voice.model_spec_id == MOSS_NANO_MODEL_SPEC_ID => {
            crate::spaces::speech::preset_voices::require(
                voice
                    .preset_voice_id
                    .as_deref()
                    .ok_or("Preset speaker is unavailable")?,
                &voice.language,
            )?;
        }
        ("moss_nano", "reference")
            if voice.model_spec_id == MOSS_NANO_MODEL_SPEC_ID
                && recorded
                && voice.reference_text.is_none()
                && voice.design_description.is_none() => {}
        ("voxcpm2", "reference" | "designed")
            if model_engine == Some("voxcpm2")
                && recorded
                && (voice.source_kind == "reference"
                    || voice
                        .reference_text
                        .as_deref()
                        .is_some_and(|text| !text.trim().is_empty())) => {}
        (engine @ ("qwen3_tts" | "cosyvoice3"), "reference")
            if model_engine == Some(engine)
                && recorded
                && voice
                    .reference_text
                    .as_deref()
                    .is_none_or(|text| !text.trim().is_empty()) => {}
        ("index_tts2", "reference")
            if model_engine == Some("index_tts2") && recorded && voice.reference_text.is_none() => {
        }
        _ => return Err("Voice revision has an unsupported production configuration".into()),
    }
    let config = parse_config(&voice.synthesis_config)?;
    if config.sample_rate != 48_000 || config.speed != 1.0 {
        return Err("Voice revision requires the fixed 48 kHz normal-speed profile".into());
    }
    Ok(())
}

/// How a line with a video window is brought to its time before the mix compresses what is left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::spaces::speech) enum DurationFit {
    /// The engine generates the line to the window's length: one natural take measures it, and
    /// one take at the scale that fits replaces it when it overran.
    Scale,
    /// The engine speaks at its own pace, so up to this many takes over procedurally derived
    /// seeds (`base_seed.wrapping_add(take)`) look for one that happens to fit. Reproducible from
    /// the cached identity.
    Takes(u32),
}

impl DurationFit {
    pub(in crate::spaces::speech) fn of(engine_id: &str) -> Result<Self, String> {
        let engine = crate::ai::contracts::speech_wire::SpeechSynthesisEngine::from_id(engine_id)
            .ok_or("Voice revision engine is unsupported")?;
        Ok(if engine.controls_duration() {
            Self::Scale
        } else {
            Self::Takes(3)
        })
    }
}

pub(in crate::spaces::speech) fn input_digest(
    segment: &SpeechSegmentExecution,
    voice: &VoiceExecution,
    config: &VoiceSynthesisConfig,
    runtime_digest: &str,
    execution_profile_id: &str,
    fit: Option<(i64, i64)>,
) -> Result<String, String> {
    let mut value = serde_json::json!({
        "text": segment.normalized_text,
        "deliveryInstruction": segment.delivery_instruction,
        "voiceRevisionId": voice.revision_id,
        "engineId": voice.engine_id,
        "modelSpecId": voice.model_spec_id,
        "language": segment.language,
        "sourceRevision": voice.reference_source_revision,
        "synthesis": config,
        "executionProfileId": execution_profile_id,
        "seed": config.seed,
        "engineVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest,
        "preprocessingVersion": "speech-text-v1",
        "renderVersion": "speech-render-v2-trimmed",
    });
    if let Some((start_ms, end_ms)) = fit {
        value["durationFit"] = match DurationFit::of(&voice.engine_id)? {
            DurationFit::Takes(takes) => serde_json::json!({
                "targetStartMs": start_ms,
                "targetEndMs": end_ms,
                "takes": takes,
            }),
            DurationFit::Scale => serde_json::json!({
                "targetStartMs": start_ms,
                "targetEndMs": end_ms,
                "minimumScale": crate::ai::contracts::speech_wire::MIN_DURATION_SCALE,
            }),
        };
    }
    Ok(hex::encode(Sha256::digest(
        serde_json::to_vec(&value).map_err(db::json_error)?,
    )))
}
