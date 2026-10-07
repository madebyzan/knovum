use super::{kokoro_device_profile, parse_config, KOKORO_MODEL_SPEC_ID};
use crate::spaces::{
    db,
    speech::contracts::{SpeechSegmentExecution, VoiceExecution, VoiceSynthesisConfig},
};
use sha2::{Digest, Sha256};

pub(in crate::spaces::speech) async fn load_voice(
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
        ("kokoro", "preset") if voice.model_spec_id == KOKORO_MODEL_SPEC_ID => {
            crate::spaces::speech::preset_voices::require(
                voice
                    .preset_voice_id
                    .as_deref()
                    .ok_or("Preset speaker is unavailable")?,
                &voice.language,
            )?;
        }
        ("voxcpm2", "reference" | "designed")
            if voice.model_spec_id == crate::spaces::speech::voice_candidates::MODEL_SPEC_ID
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

pub(in crate::spaces::speech) fn input_digest(
    segment: &SpeechSegmentExecution,
    voice: &VoiceExecution,
    config: &VoiceSynthesisConfig,
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
        "seed": config.seed,
        "engineVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest,
        "preprocessingVersion": "speech-text-v1",
    });
    Ok(hex::encode(Sha256::digest(
        serde_json::to_vec(&value).map_err(db::json_error)?,
    )))
}
