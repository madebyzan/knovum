use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Runtime};

use crate::foundation::app_error::AppError;
use crate::spaces::knowledge::retrieval::{self, rebuild::KnowledgeIndexingProgress};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalIndexOutput {
    indexed: usize,
}

/// Indexing a Space is long Rust work, so it is a job: `job_id` names it for the Task Center,
/// `on_progress` streams what it has encoded, and `cancel_knowledge_indexing` pulls its token.
#[tauri::command]
pub async fn index_knowledge<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
    job_id: String,
    on_progress: Channel<KnowledgeIndexingProgress>,
) -> Result<KnowledgeRetrievalIndexOutput, AppError> {
    match retrieval::index_space(app, space_id, node_ids, job_id, &mut |progress| {
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
        Err(error) => Err(error.into()),
    }
}

#[tauri::command]
pub fn cancel_knowledge_indexing(job_id: String) -> Result<(), AppError> {
    retrieval::indexing_control::cancel(&job_id)?;
    Ok(())
}

#[tauri::command]
pub fn cancel_knowledge_search(job_id: String) -> Result<(), AppError> {
    Ok(retrieval::search_control::cancel(&job_id)?)
}
