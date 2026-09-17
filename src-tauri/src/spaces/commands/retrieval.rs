use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::spaces::knowledge::retrieval;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalRebuildOutput {
    rebuilt: bool,
    indexed: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalIndexOutput {
    indexed: usize,
}

#[tauri::command]
pub async fn index_knowledge<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    node_ids: Option<Vec<String>>,
) -> Result<KnowledgeRetrievalIndexOutput, String> {
    let indexed = retrieval::index_space(app, space_id, node_ids)
        .await?
        .indexed;
    Ok(KnowledgeRetrievalIndexOutput { indexed })
}

#[tauri::command]
pub async fn rebuild_knowledge_retrieval_index<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<KnowledgeRetrievalRebuildOutput, String> {
    let retrieval::rebuild::RebuildIndexOutput { rebuilt, indexed } =
        retrieval::rebuild_space_index(app, space_id).await?;
    Ok(KnowledgeRetrievalRebuildOutput { rebuilt, indexed })
}
