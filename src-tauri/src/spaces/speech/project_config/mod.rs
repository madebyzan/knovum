mod audiobook;

use serde::Deserialize;

pub(crate) use audiobook::{
    parse as parse_audiobook, role_voice, validate_chapter_order, validate_role_bindings,
    AudiobookCover, AudiobookMetadata, AudiobookOutputConfig, AudiobookRoleBinding,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ShortOutputConfig {
    pub format: String,
    pub wav_precision: super::audio_format::WavPrecision,
}

pub(crate) fn validate(project_type: &str, value: &serde_json::Value) -> Result<(), String> {
    match project_type {
        "short_dubbing" => {
            let config: ShortOutputConfig =
                serde_json::from_value(value.clone()).map_err(crate::spaces::db::json_error)?;
            if config.format != "wav" {
                return Err("Short dubbing requires WAV output".to_owned());
            }
            Ok(())
        }
        "audiobook" => parse_audiobook(value).map(|_| ()),
        "video_dubbing" => super::video_config::parse(value).map(|_| ()),
        _ => Err("This speech project type has no complete production workflow".to_owned()),
    }
}
