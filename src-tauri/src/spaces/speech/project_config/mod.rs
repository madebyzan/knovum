mod audiobook;

use serde::{Deserialize, Serialize};

pub(crate) use audiobook::{
    parse as parse_audiobook, role_voice, validate_chapter_order, validate_role_bindings,
    AudiobookCover, AudiobookMetadata, AudiobookOutputConfig, AudiobookRoleBinding,
};

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ShortOutputConfig {
    pub format: String,
    pub wav_precision: super::audio_format::WavPrecision,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(untagged)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ProjectOutputConfig {
    ShortDubbing(ShortOutputConfig),
    Audiobook(AudiobookOutputConfig),
    VideoDubbing(super::video_config::VideoDubbingOutputConfig),
}

impl ProjectOutputConfig {
    pub(crate) fn validate(&self, project_type: &str) -> Result<(), String> {
        match (project_type, self) {
            ("short_dubbing", Self::ShortDubbing(config)) => {
                if config.format != "wav" {
                    return Err("Short dubbing requires WAV output".to_owned());
                }
                Ok(())
            }
            ("audiobook", Self::Audiobook(config)) => audiobook::validate(config),
            ("video_dubbing", Self::VideoDubbing(config)) => super::video_config::validate(config),
            _ => Err("Speech output configuration does not match the project type".into()),
        }
    }
}

pub(crate) fn parse(
    project_type: &str,
    value: &serde_json::Value,
) -> Result<ProjectOutputConfig, String> {
    let config = match project_type {
        "short_dubbing" => ProjectOutputConfig::ShortDubbing(
            ShortOutputConfig::deserialize(value).map_err(crate::spaces::db::json_error)?,
        ),
        "audiobook" => ProjectOutputConfig::Audiobook(
            AudiobookOutputConfig::deserialize(value).map_err(crate::spaces::db::json_error)?,
        ),
        "video_dubbing" => ProjectOutputConfig::VideoDubbing(
            super::video_config::VideoDubbingOutputConfig::deserialize(value)
                .map_err(crate::spaces::db::json_error)?,
        ),
        _ => return Err("This speech project type has no complete production workflow".into()),
    };
    config.validate(project_type)?;
    Ok(config)
}
