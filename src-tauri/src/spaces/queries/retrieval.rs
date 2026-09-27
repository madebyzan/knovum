use crate::command_error::CommandError;
use tauri::{AppHandle, Runtime};

use crate::spaces::knowledge::retrieval;

#[tauri::command]
pub async fn search_knowledge_hybrid<R: Runtime>(
    app: AppHandle<R>,
    job_id: String,
    request: retrieval::hybrid::HybridSearchRequest,
) -> Result<retrieval::hybrid::HybridSearchResponse, CommandError> {
    let (cancellation, _lease) = searches().acquire_cancellable(job_id)?;
    Ok(tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err("knowledge-search-cancelled".to_owned()),
        result = retrieval::hybrid::search(&app, request, &cancellation) => result,
    }?)
}

#[tauri::command]
pub async fn list_related_content<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_id: String,
) -> Result<retrieval::related::RelatedContentResponse, CommandError> {
    Ok((retrieval::related::list(&app, &space_id, &node_id).await)?)
}

fn searches() -> &'static crate::job_registry::JobRegistry {
    static SEARCHES: std::sync::OnceLock<crate::job_registry::JobRegistry> =
        std::sync::OnceLock::new();
    SEARCHES.get_or_init(|| crate::job_registry::JobRegistry::new("Knowledge search"))
}

#[tauri::command]
pub fn cancel_knowledge_search(job_id: String) -> Result<(), CommandError> {
    Ok(searches().cancel(&job_id)?)
}
