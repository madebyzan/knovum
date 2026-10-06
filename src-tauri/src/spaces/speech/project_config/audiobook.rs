use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::super::contracts::SpeechSegmentInput;

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechChapter {
    pub key: String,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct AudiobookRoleBinding {
    pub key: String,
    pub voice_revision_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct AudiobookOutputConfig {
    pub format: String,
    pub sample_rate: u32,
    pub loudness_lufs: f32,
    pub true_peak_db: f32,
    pub chapters: Vec<SpeechChapter>,
    pub roles: Vec<AudiobookRoleBinding>,
    pub paragraph_pause_ms: u32,
    pub chapter_pause_ms: u32,
    pub wav_precision: super::super::audio_format::WavPrecision,
    pub metadata: AudiobookMetadata,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct AudiobookMetadata {
    pub title: String,
    pub author: String,
    pub narrator: String,
    pub cover: Option<AudiobookCover>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct AudiobookCover {
    pub asset_path: String,
    pub source_revision: String,
}

pub(crate) fn parse(value: &serde_json::Value) -> Result<AudiobookOutputConfig, String> {
    let config =
        AudiobookOutputConfig::deserialize(value).map_err(crate::spaces::db::json_error)?;
    validate(&config)?;
    Ok(config)
}

pub(super) fn validate(config: &AudiobookOutputConfig) -> Result<(), String> {
    if !matches!(config.format.as_str(), "wav" | "mp3" | "m4b")
        || !matches!(config.sample_rate, 24_000 | 48_000)
        || !config.loudness_lufs.is_finite()
        || !(-24.0..=-16.0).contains(&config.loudness_lufs)
        || !config.true_peak_db.is_finite()
        || !(-3.0..=-1.0).contains(&config.true_peak_db)
        || config.chapters.len() > 10_000
        || config.roles.len() > 1_000
    {
        return Err("Audiobook output configuration is invalid".to_owned());
    }
    if config.paragraph_pause_ms > 10_000
        || config.chapter_pause_ms > 30_000
        || [
            &config.metadata.title,
            &config.metadata.author,
            &config.metadata.narrator,
        ]
        .into_iter()
        .any(|value| value.chars().count() > 200)
    {
        return Err("Audiobook pause or metadata exceeds its supported bounds".into());
    }
    if let Some(cover) = &config.metadata.cover {
        if config.format == "wav" {
            return Err("WAV audiobook output cannot embed a book cover; choose MP3 or M4B".into());
        }
        if cover.asset_path.is_empty()
            || cover.source_revision.len() != 71
            || !cover.source_revision.starts_with("sha256:")
            || !cover.source_revision[7..]
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "Audiobook cover requires a managed asset and exact content revision".into(),
            );
        }
    }
    validate_chapters(&config.chapters)?;
    validate_roles(&config.roles)?;
    Ok(())
}

fn validate_chapters(chapters: &[SpeechChapter]) -> Result<(), String> {
    let mut keys = HashSet::new();
    for chapter in chapters {
        uuid::Uuid::parse_str(&chapter.key).map_err(|_| "Chapter identity must be a UUID")?;
        let length = chapter.title.trim().chars().count();
        if !(1..=200).contains(&length) || !keys.insert(&chapter.key) {
            return Err(
                "Audiobook chapters require unique identities and nonempty titles".to_owned(),
            );
        }
    }
    Ok(())
}

fn validate_roles(roles: &[AudiobookRoleBinding]) -> Result<(), String> {
    let mut keys = HashSet::new();
    for role in roles {
        let key = role.key.trim();
        if !(1..=120).contains(&key.chars().count()) || key != role.key || !keys.insert(key) {
            return Err("Audiobook roles require unique normalized names".to_owned());
        }
        uuid::Uuid::parse_str(&role.voice_revision_id)
            .map_err(|_| "Audiobook role voice revision must be a UUID")?;
    }
    Ok(())
}

pub(crate) fn validate_role_bindings(
    config: &AudiobookOutputConfig,
    segments: &[SpeechSegmentInput],
) -> Result<(), String> {
    for segment in segments {
        let voice = role_voice(config, segment.speaker_key.as_deref())?;
        if voice != segment.voice_revision_id {
            return Err("Audiobook paragraph voice differs from its role binding".to_owned());
        }
    }
    Ok(())
}

pub(crate) fn role_voice<'a>(
    config: &'a AudiobookOutputConfig,
    speaker_key: Option<&str>,
) -> Result<&'a str, String> {
    let speaker = speaker_key.ok_or("Every audiobook paragraph must reference a role")?;
    config
        .roles
        .iter()
        .find(|role| role.key == speaker)
        .map(|role| role.voice_revision_id.as_str())
        .ok_or_else(|| "Audiobook paragraph references an unavailable role".to_owned())
}

pub(crate) fn validate_chapter_order<'a>(
    config: &AudiobookOutputConfig,
    chapter_keys: impl Iterator<Item = Option<&'a str>>,
) -> Result<(), String> {
    let mut sequence = Vec::new();
    for key in chapter_keys {
        let key = key.ok_or("Every audiobook paragraph must belong to a chapter")?;
        if sequence.last().copied() != Some(key) {
            sequence.push(key);
        }
    }
    if sequence.len() != config.chapters.len()
        || sequence
            .iter()
            .zip(&config.chapters)
            .any(|(key, chapter)| *key != chapter.key)
    {
        return Err(
            "Audiobook paragraphs must follow the configured chapter order without empty chapters"
                .to_owned(),
        );
    }
    Ok(())
}
