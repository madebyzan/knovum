use serde_json::{Map, Value};
use std::path::Path;

use super::paths;

const KEYS: &[&str] = &[
    "preferences",
    "ai.models",
    "creation.library",
    "localResources.transcription",
    "localResources.speechSynthesis",
    "localResources.imageSegmentation",
    "localResources.imageUpscaling",
    "localResources.imageInpainting",
    "localResources.imageSelection",
    "localResources.imageDepth",
    "localResources.speechSeparation",
    "localResources.speechDiarization",
    "localResources.speechAlignment",
];

pub fn validate(path: &Path) -> Result<(), String> {
    let document: Value = serde_json::from_slice(
        &paths::read_bounded(path, 16 * 1024 * 1024)
            .map_err(|error| format!("Settings could not be read: {error}"))?,
    )
    .map_err(|error| format!("Settings JSON is invalid: {error}"))?;
    let object = document
        .as_object()
        .ok_or("Settings root must be an object")?;
    if object.keys().any(|key| !KEYS.contains(&key.as_str())) {
        return Err("Settings contain an unsupported key".into());
    }
    for (key, value) in object {
        let envelope = value
            .as_object()
            .ok_or_else(|| format!("{key} must be an envelope"))?;
        require_envelope(envelope, key)?;
        validate_value(
            key,
            envelope
                .get("value")
                .ok_or_else(|| format!("{key} has no value"))?,
        )?;
    }
    Ok(())
}

fn require_envelope(envelope: &Map<String, Value>, key: &str) -> Result<(), String> {
    if envelope.len() != 2
        || !envelope.contains_key("schemaVersion")
        || !envelope.contains_key("value")
    {
        return Err(format!("{key} has an invalid envelope"));
    }
    let version = if key == "preferences" {
        crate::preferences::SCHEMA_VERSION as u64
    } else {
        1
    };
    if envelope.get("schemaVersion").and_then(Value::as_u64) != Some(version) {
        return Err(format!("{key} uses an unsupported schema"));
    }
    Ok(())
}

fn validate_value(key: &str, value: &Value) -> Result<(), String> {
    if key == "preferences" {
        let preferences = serde_json::from_value::<crate::preferences::Preferences>(value.clone())
            .map_err(|error| format!("Preferences do not match the current schema: {error}"))?;
        return preferences.validate();
    }
    if key.starts_with("localResources.") {
        return crate::local_resources::model_selection::validate_persisted_value(
            key,
            value.clone(),
        )
        .map_err(|error| format!("{key} contains an invalid model identifier: {error}"));
    }
    if key == "ai.models" {
        let settings = serde_json::from_value::<crate::ai::settings::AiSettings>(value.clone())
            .map_err(|error| format!("AI model settings are invalid: {error}"))?;
        crate::ai::settings::validate_model_settings(settings).map_err(|error| error.message)?;
    }
    Ok(())
}
