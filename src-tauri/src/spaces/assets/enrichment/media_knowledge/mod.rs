pub(crate) mod insights;

use crate::{
    command_error::CommandError, spaces::assets::transcription::MediaTranscriptionProgress,
};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

pub(crate) fn validate_input(current: &Value, expected: &Value) -> Result<(), String> {
    if current != expected {
        return Err(
            "Media knowledge changed while processing; the generated result was not saved".into(),
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn generate_media_insights(
    app: AppHandle,
    space_id: String,
    node_id: String,
    job_id: String,
    expected_revision: String,
    on_progress: Channel<MediaTranscriptionProgress>,
    target: Option<crate::ai::tasks::TaskTargetSelection>,
) -> Result<Value, CommandError> {
    let (cancellation, _lease) = crate::ai::executions::acquire(job_id)?;
    let data = super::node_payload::load_node_content(&app, &space_id, &node_id).await?;
    if data["knowledgeRevision"].as_str() != Some(&expected_revision) {
        return Err("Media knowledge changed before processing began".into());
    }
    let source = super::node_payload::required_string(&data, "sourceRevision")?;
    let knowledge = data
        .get("mediaKnowledge")
        .filter(|value| value["sourceRevision"].as_str() == Some(&source))
        .ok_or("Current media transcript is unavailable")?
        .clone();
    let generated_insights =
        insights::generate(&app, target, &knowledge, &on_progress, &cancellation).await;
    if cancellation.is_cancelled() {
        return Err(CommandError::new(
            "media_insights_cancelled",
            "media-insights-cancelled: Media insights generation cancelled",
        ));
    }
    let next = generated_insights?;
    let fields = serde_json::Map::from_iter([("mediaKnowledge".into(), next)]);
    Ok(super::commit::publish(
        &app,
        &space_id,
        &node_id,
        super::commit::PublicationGuard {
            source_revision: Some(&source),
            job_id: None,
            media_knowledge: Some(&knowledge),
            document_coverage: None,
            cancellation: Some(&cancellation),
        },
        fields,
        None,
    )
    .await?)
}
