use sqlx::{Sqlite, Transaction};

use super::project_config::ProjectOutputConfig;
use crate::spaces::db;

pub(crate) fn validate_kind(kind: &str, project_type: &str) -> Result<(), String> {
    let allowed = match project_type {
        "short_dubbing" => matches!(kind, "note" | "text" | "markdown"),
        "audiobook" => matches!(kind, "note" | "text" | "markdown" | "file"),
        "video_dubbing" => kind == "video",
        _ => false,
    };
    if allowed {
        Ok(())
    } else {
        Err("Speech operation does not support this source node".into())
    }
}

pub(super) async fn require(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    config: &ProjectOutputConfig,
) -> Result<(), String> {
    let (kind, data): (String, String) = sqlx::query_as(
        "SELECT kind,data FROM spaces_nodes WHERE id=? AND space_id=? AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Speech source node is not available in this Space")?;
    let project_type = match config {
        ProjectOutputConfig::ShortDubbing(_) => "short_dubbing",
        ProjectOutputConfig::Audiobook(_) => "audiobook",
        ProjectOutputConfig::VideoDubbing(_) => "video_dubbing",
    };
    validate_kind(&kind, project_type)?;
    if let ProjectOutputConfig::VideoDubbing(config) = config {
        let data: serde_json::Value = serde_json::from_str(&data).map_err(db::json_error)?;
        if data["assetPath"].as_str() != Some(config.video.asset_path.as_str())
            || data["sourceRevision"].as_str() != Some(config.video.source_revision.as_str())
        {
            return Err("Video dubbing source differs from its source node".into());
        }
    }
    Ok(())
}
