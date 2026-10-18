use sqlx::{Sqlite, Transaction};

use super::super::{
    contracts::SpeechAudioOutput,
    publication::nodes::{self, MediaNodeOutput},
};
use crate::spaces::{db, derivation::inheritance::DerivedOutput, previews::GeneratedPreview};

#[cfg(test)]
mod tests;

/// The node a speech project Run reserved for its output.
pub(in crate::spaces::speech) fn output_node_id(run_id: &str) -> String {
    format!("{run_id}-output")
}

pub(super) async fn settle(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: &str,
    run_id: &str,
    output: &SpeechAudioOutput,
    cover: Option<GeneratedPreview>,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    let (space_id, source_node_id, project_type): (String, String, String) = sqlx::query_as(
        "SELECT p.space_id,p.source_node_id,p.project_type FROM spaces_speech_projects p \
         JOIN spaces_nodes n ON n.id=p.source_node_id AND n.space_id=p.space_id \
         WHERE p.id=? AND n.deleted_at IS NULL",
    )
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Speech source node is no longer available")?;
    // Dubbing gives a video a new voice; every other project narrates a text into new audio.
    let derived = if project_type == "video_dubbing" {
        DerivedOutput::DubbedVideo
    } else {
        DerivedOutput::Narration
    };
    nodes::settle(
        tx,
        &space_id,
        &source_node_id,
        &[MediaNodeOutput {
            node_id: output_node_id(run_id),
            output: output.clone(),
            derived,
            cover,
        }],
        enrichment_target,
    )
    .await
}
