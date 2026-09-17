use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::{db, knowledge::grounding};

use super::answer_request::GroundedKnowledgeRequest;
use super::contracts::GroundedThreadCreationRequest;

pub(super) struct Context {
    pub(super) scope: String,
    pub(super) node_ids: Vec<String>,
    pub(super) history: Vec<(String, String)>,
    pub(super) existing: bool,
}

impl Context {
    pub(super) fn new_thread(&self, question: &str) -> Option<GroundedThreadCreationRequest> {
        (!self.existing).then(|| GroundedThreadCreationRequest {
            scope: self.scope.clone(),
            node_ids: self.node_ids.clone(),
            title: crate::utf16::truncate_with_suffix(question, 96, ""),
        })
    }
}

pub(super) async fn load_context(
    app: &AppHandle,
    request: &GroundedKnowledgeRequest,
) -> Result<Context, String> {
    if let Some(thread_id) = request.thread_id.as_deref() {
        let pool = db::pool(app).await?;
        let mut tx = pool.begin().await.map_err(db::sql_error)?;
        let contract = grounding::thread_contract(&mut tx, thread_id).await?;
        if contract.space_id != request.space_id {
            return Err("Knowledge thread belongs to another Space".to_owned());
        }
        tx.commit().await.map_err(db::sql_error)?;
        let rows = sqlx::query(
            "SELECT role,content FROM spaces_ai_messages WHERE thread_id=$1 ORDER BY created_at DESC,id DESC LIMIT 8",
        )
        .bind(thread_id)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
        let mut history = rows
            .into_iter()
            .map(|row| {
                Ok((
                    row.try_get("role").map_err(db::sql_error)?,
                    row.try_get("content").map_err(db::sql_error)?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        history.reverse();
        return Ok(Context {
            scope: contract.scope,
            node_ids: contract.node_ids,
            history,
            existing: true,
        });
    }
    grounding::validate_scope_shape(&request.scope, &request.node_ids)?;
    Ok(Context {
        scope: request.scope.clone(),
        node_ids: request.node_ids.clone(),
        history: Vec::new(),
        existing: false,
    })
}
