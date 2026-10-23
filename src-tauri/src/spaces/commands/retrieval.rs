use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Runtime};

use crate::foundation::app_error::AppError;
use crate::spaces::knowledge::retrieval::{
    self, index_state::KnowledgeIndexState, rebuild::KnowledgeIndexingProgress,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalIndexOutput {
    indexed: usize,
}

/// What a search can reach in this Space today, and what indexing it would still have to do.
#[tauri::command]
pub async fn knowledge_index_state<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<KnowledgeIndexState, AppError> {
    Ok(retrieval::index_state::read(&app, &space_id).await?)
}

/// Indexing a Space is long Rust work and a Space has one pass at a time, so the Space names it:
/// `on_progress` streams what it has encoded, and `cancel_knowledge_indexing` pulls its token.
#[tauri::command]
pub async fn index_knowledge<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
    on_progress: Channel<KnowledgeIndexingProgress>,
) -> Result<KnowledgeRetrievalIndexOutput, AppError> {
    match retrieval::index_space(app, space_id, node_ids, &mut |progress| {
        let _ = on_progress.send(progress);
    })
    .await
    {
        Ok(output) => Ok(KnowledgeRetrievalIndexOutput {
            indexed: output.indexed,
        }),
        // A pulled token is an outcome, not a failure: the encoded batches are committed and the
        // live index never went down, so the next pass resumes from there.
        Err(error) if error == retrieval::rebuild::CANCELLED => {
            Err(AppError::cancelled("Knowledge indexing was cancelled"))
        }
        // So is finding the Space already being indexed: the work this call asked for is in flight.
        Err(error) if error == retrieval::indexing_control::BUSY => Err(AppError::new(
            "knowledge_indexing_busy",
            "This space is already being indexed",
        )),
        Err(error) => Err(error.into()),
    }
}

#[tauri::command]
pub fn cancel_knowledge_indexing(space_id: String) -> Result<(), AppError> {
    retrieval::indexing_control::cancel(&space_id)?;
    Ok(())
}

#[tauri::command]
pub fn cancel_knowledge_search(job_id: String) -> Result<(), AppError> {
    Ok(retrieval::search_control::cancel(&job_id)?)
}
