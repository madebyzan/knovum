use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct PresetVoice {
    pub speaker_id: String,
    pub name: String,
    pub native_language: String,
    pub gender: String,
}

pub(super) fn catalog() -> Vec<PresetVoice> {
    serde_json::from_str(include_str!("preset-voices.json")).expect("Owned Kokoro voice catalog")
}

pub(crate) fn require(speaker_id: &str, language: &str) -> Result<PresetVoice, String> {
    if !matches!(language, "zh-CN" | "en") {
        return Err("Kokoro requires a supported Chinese or English language profile".into());
    }
    catalog()
        .into_iter()
        .find(|voice| voice.speaker_id == speaker_id)
        .ok_or_else(|| "Kokoro voice is absent from the pinned model catalog".into())
}

#[tauri::command]
pub fn list_preset_voices() -> Vec<PresetVoice> {
    catalog()
}
