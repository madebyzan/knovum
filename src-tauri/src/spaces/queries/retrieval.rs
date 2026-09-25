use tauri::{AppHandle, Runtime};

use crate::spaces::knowledge::retrieval;

#[tauri::command]
pub async fn search_knowledge_hybrid<R: Runtime>(
    app: AppHandle<R>,
    request: retrieval::hybrid::HybridSearchRequest,
) -> Result<retrieval::hybrid::HybridSearchResponse, String> {
    retrieval::hybrid::search(&app, request).await
}

#[tauri::command]
pub async fn list_related_content<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_id: String,
) -> Result<retrieval::related::RelatedContentResponse, String> {
    retrieval::related::list(&app, &space_id, &node_id).await
}
