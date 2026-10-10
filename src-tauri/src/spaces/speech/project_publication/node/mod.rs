use sqlx::{Sqlite, Transaction};

use super::super::{
    contracts::SpeechAudioOutput,
    publication::nodes::{self, MediaNodeOutput},
};
use crate::spaces::db;
#[cfg(test)]
mod tests;

pub(super) async fn publish(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: &str,
    revision_id: &str,
    output: &SpeechAudioOutput,
) -> Result<(), String> {
    let (space_id, source_node_id, title, project_type): (String, String, String, String) = sqlx::query_as(
        "SELECT p.space_id,p.source_node_id,p.title,p.project_type FROM spaces_speech_projects p JOIN spaces_nodes n ON n.id=p.source_node_id AND n.space_id=p.space_id WHERE p.id=? AND n.deleted_at IS NULL",
    ).bind(project_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
        .ok_or("Speech source node is no longer available")?;
    let node_id = format!("{revision_id}-output");
    let existing_content_json: Option<String> =
        sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=? AND space_id=?")
            .bind(&node_id)
            .bind(&space_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    if let Some(node_content_json) = existing_content_json {
        let content: serde_json::Value =
            serde_json::from_str(&node_content_json).map_err(db::json_error)?;
        if content["assetPath"].as_str() != Some(&output.path)
            || content["sourceRevision"] != format!("sha256:{}", output.sha256)
        {
            return Err("Speech revision already published a different node output".into());
        }
        return Ok(());
    }
    let kind = if project_type == "video_dubbing" {
        "video"
    } else {
        "audio"
    };
    nodes::publish(
        tx,
        &space_id,
        &source_node_id,
        &[MediaNodeOutput {
            node_id,
            kind,
            title,
            output: output.clone(),
        }],
    )
    .await
}
