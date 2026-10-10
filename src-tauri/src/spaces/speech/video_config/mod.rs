use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

mod separation;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechMediaSource {
    pub asset_path: String,
    pub source_revision: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VideoDubbingOutputConfig {
    pub format: String,
    pub video: SpeechMediaSource,
    pub background: Option<SpeechMediaSource>,
    pub background_separation: Option<SourceSeparationProvenance>,
    pub voice_gain_db: f32,
    pub background_gain_db: f32,
    pub translation: Option<VideoTranslationProvenance>,
    pub preserve_original_outside_dialogue: bool,
    pub source_dialogue_intervals: Vec<SourceDialogueInterval>,
    pub retain_original_track: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SourceDialogueInterval {
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SourceSeparationProvenance {
    pub separation_id: String,
    pub model_spec_id: String,
    pub source: SpeechMediaSource,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VideoTranslationProvenance {
    pub run_id: String,
    pub target_language: String,
    pub provider: String,
    pub model: String,
    pub local_only: bool,
}

pub(crate) fn parse(value: &serde_json::Value) -> Result<VideoDubbingOutputConfig, String> {
    let config = VideoDubbingOutputConfig::deserialize(value).map_err(db::json_error)?;
    validate(&config)?;
    Ok(config)
}

pub(crate) fn validate(config: &VideoDubbingOutputConfig) -> Result<(), String> {
    if config.format != "mp4"
        || !valid_gain(config.voice_gain_db)
        || !valid_gain(config.background_gain_db)
    {
        return Err("Video dubbing requires MP4 output and gains between -30 and 6 dB".into());
    }
    for source in std::iter::once(&config.video).chain(config.background.iter()) {
        if source.asset_path.trim().is_empty()
            || source.duration_ms == 0
            || source.duration_ms > 43_200_000
            || !valid_source_revision(&source.source_revision)
        {
            return Err("Video dubbing sources require a managed asset, SHA-256 revision and duration up to twelve hours".into());
        }
    }
    if config.source_dialogue_intervals.len() > 10_000
        || config.source_dialogue_intervals.iter().any(|window| {
            window.end_ms <= window.start_ms || window.end_ms > config.video.duration_ms
        })
        || (config.preserve_original_outside_dialogue
            && (config.background_separation.is_none()
                || config.source_dialogue_intervals.is_empty()))
    {
        return Err("Preserving original ambience requires source dialogue intervals and a published separation".into());
    }
    separation::validate(config)?;
    if let Some(translation) = &config.translation {
        crate::ai::capabilities::translation::validate_target(&translation.target_language)
            .map_err(|error| error.to_string())?;
        if !canonical_uuid(&translation.run_id)
            || translation.provider.trim().is_empty()
            || translation.provider.encode_utf16().count() > 128
            || translation.model.trim().is_empty()
            || translation.model.encode_utf16().count() > 256
            || (translation.local_only && translation.provider != "ollama")
        {
            return Err("Video translation provenance is invalid".into());
        }
    }
    Ok(())
}

fn valid_gain(gain: f32) -> bool {
    gain.is_finite() && (-30.0..=6.0).contains(&gain)
}

fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|parsed| parsed.to_string() == value)
}

fn valid_source_revision(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A source is owned by a live Space node or an immutable revision of this same project.
/// Historical ownership allows editing a project after its source node has been removed.
pub(crate) async fn require_sources(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    project_id: Option<&str>,
    config: &VideoDubbingOutputConfig,
) -> Result<(), String> {
    require_source(tx, space_id, project_id, "video", &config.video).await?;
    if let Some(background) = &config.background {
        if let Some(separation) = &config.background_separation {
            separation::require_published(tx, space_id, separation, background).await?;
        } else {
            require_source(tx, space_id, project_id, "audio", background).await?;
        }
    }
    Ok(())
}

pub(crate) async fn require_source(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    project_id: Option<&str>,
    kind: &str,
    source: &SpeechMediaSource,
) -> Result<(), String> {
    let owned = super::media_source::is_owned(tx, space_id, &[kind], source).await?;
    if owned {
        return Ok(());
    }
    if let Some(project_id) = project_id {
        let configs: Vec<String> = sqlx::query_scalar(
            "SELECT r.output_config FROM spaces_speech_project_revisions r \
             JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.id=? AND p.space_id=? AND p.project_type='video_dubbing'",
        ).bind(project_id).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
        for value in configs {
            let config = parse(&serde_json::from_str(&value).map_err(db::json_error)?)?;
            let previous = if kind == "video" {
                Some(&config.video)
            } else {
                config.background.as_ref()
            };
            if previous == Some(source) {
                return Ok(());
            }
        }
    }
    Err("Video dubbing source is not an analyzed media asset owned by this Space or project".into())
}

pub(crate) fn validate_timing(
    config: &VideoDubbingOutputConfig,
    start: Option<i64>,
    end: Option<i64>,
    chapter: Option<&str>,
) -> Result<(), String> {
    match (start, end) {
        (Some(start), Some(end)) if chapter.is_none() && start >= 0 && end > start
            && u64::try_from(end).is_ok_and(|end| end <= config.video.duration_ms) => Ok(()),
        _ => Err("Video dubbing segments require valid time windows within the source video and no chapter".into()),
    }
}

#[cfg(test)]
#[path = "tests/separation.rs"]
mod tests;
