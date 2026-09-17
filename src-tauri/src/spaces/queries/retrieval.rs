use tauri::{AppHandle, Runtime};

use crate::spaces::knowledge::retrieval;

#[tauri::command]
pub async fn search_knowledge_hybrid<R: Runtime>(
    app: AppHandle<R>,
    request: retrieval::hybrid::HybridSearchRequest,
) -> Result<retrieval::hybrid::HybridSearchResponse, String> {
    retrieval::hybrid::search(&app, request).await
}
