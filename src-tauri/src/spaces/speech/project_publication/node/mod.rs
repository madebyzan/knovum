use serde_json::json;
use sqlx::{Sqlite, Transaction};

use super::super::contracts::SpeechAudioOutput;
#[cfg(test)]
mod tests;
use crate::spaces::{
    db,
    nodes::{
        self,
        contracts::{InsertNode, InsertNodesRequest},
    },
    structure::{self, SizingMode},
};

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
    let existing: Option<String> =
        sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=? AND space_id=?")
            .bind(&node_id)
            .bind(&space_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    if let Some(data) = existing {
        let data: serde_json::Value = serde_json::from_str(&data).map_err(db::json_error)?;
        if data["assetPath"].as_str() != Some(&output.path)
            || data["sourceRevision"] != format!("sha256:{}", output.sha256)
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
    let path = std::path::Path::new(&output.path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Speech output has no file name")?;
    let (snapshot, edit_revision) =
        structure::repository::load_in_transaction(tx, &space_id).await?;
    let (x, y) = structure::place_new_content(
        &snapshot.structure,
        &snapshot.layouts,
        &[source_node_id],
        320.0,
        220.0,
    )?;
    let node = InsertNode {
        id: node_id,
        kind: kind.into(),
        initial_x: x,
        initial_y: y,
        width: 320.0,
        height: 220.0,
        sizing_mode: SizingMode::Content,
        appearance: None,
        file_subpath: None,
        source_chunks: Vec::new(),
        data: json!({ "kind":kind, "title":title, "assetPath":output.path, "fileName":file_name,
            "sourceRevision":format!("sha256:{}", output.sha256),
            "durationSeconds":output.sample_count / u64::from(output.sample_rate) }),
    };
    nodes::insert_many_in_transaction(
        tx,
        InsertNodesRequest {
            space_id,
            mutation_id: format!("speech-output:{revision_id}"),
            expected_edit_revision: edit_revision,
            target_parent_id: None,
            nodes: vec![node],
        },
        None,
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}
