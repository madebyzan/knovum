use crate::foundation::app_error::AppError;
use tauri::{AppHandle, Runtime};

use crate::spaces::knowledge::retrieval;

#[tauri::command]
pub async fn search_knowledge_hybrid<R: Runtime>(
    app: AppHandle<R>,
    job_id: String,
    request: retrieval::hybrid::HybridSearchRequest,
) -> Result<retrieval::hybrid::HybridSearchResponse, AppError> {
    let (cancellation, _lease) = retrieval::search_control::acquire(job_id)?;
    Ok(tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err("knowledge-search-cancelled".to_owned()),
        retrieved_chunks = retrieval::hybrid::search(&app, request, &cancellation) => retrieved_chunks,
    }?)
}

#[tauri::command]
pub async fn list_related_content<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_id: String,
) -> Result<retrieval::related::RelatedContentResponse, AppError> {
    Ok((retrieval::related::list(&app, &space_id, &node_id).await)?)
}
