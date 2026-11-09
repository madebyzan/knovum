use super::{parse_config, MOSS_NANO_MODEL_SPEC_ID};
use crate::spaces::{
    db,
    speech::contracts::{SpeechSegmentExecution, VoiceExecution, VoiceSynthesisConfig},
};
use sha2::{Digest, Sha256};

pub(in crate::spaces::speech) async fn load_voice(
    pool: &sqlx::SqlitePool,
    revision_id: &str,
) -> Result<VoiceExecution, String> {
    let voice: VoiceExecution = sqlx::query_as(
        "SELECT r.id AS revision_id,p.source_kind,r.engine_id,r.model_spec_id,r.language,          r.preset_voice_id,r.reference_asset_id,r.reference_source_revision,r.reference_text,          r.design_description,r.synthesis_config FROM spaces_voice_revisions r          JOIN spaces_voice_profiles p ON p.id=r.profile_id WHERE r.id=?",
    )
    .bind(revision_id).fetch_optional(pool).await.map_err(db::sql_error)?
    .ok_or("Voice revision is unavailable")?;
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
                && voice.reference_asset_id.is_some()
                && voice.reference_source_revision.is_some()
                && voice.reference_text.is_none()
                && voice.design_description.is_none() => {}
        ("voxcpm2", "reference" | "designed")
            if crate::local_resources::model_catalog::LocalModelId::from_voxcpm2_spec(
                &voice.model_spec_id,
            )
            .is_ok()
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
    if matches!(voice.engine_id.as_str(), "voxcpm2" | "moss_nano")
        && (config.sample_rate != 48_000 || config.speed != 1.0)
    {
        return Err("VoxCPM2 voice revision requires its fixed 48 kHz profile".into());
    }
    Ok(voice)
}

/// How many synthesis takes a duration-fitted line may spend searching for a rendering that fits
/// its video window. Take 0 uses the voice revision's frozen seed; later takes derive seeds
/// procedurally (`base_seed.wrapping_add(take)`), so the whole search stays reproducible from the
/// cached identity.
pub(in crate::spaces::speech) const DURATION_FIT_TAKES: u32 = 3;

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
        value["durationFit"] = serde_json::json!({
            "targetStartMs": start_ms,
            "targetEndMs": end_ms,
            "takes": DURATION_FIT_TAKES,
        });
    }
    Ok(hex::encode(Sha256::digest(
        serde_json::to_vec(&value).map_err(db::json_error)?,
    )))
}
