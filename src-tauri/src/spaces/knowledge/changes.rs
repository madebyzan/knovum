use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use super::change_commit::KnowledgeChange;
use crate::{foundation::app_error::AppError, spaces::db};

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct KnowledgeChangeSet {
    space_id: String,
    actor: String,
    operations: Vec<KnowledgeChange>,
}

#[tauri::command]
pub async fn commit_knowledge_changes<R: Runtime>(
    app: AppHandle<R>,
    changes: KnowledgeChangeSet,
) -> Result<Value, AppError> {
    if changes.space_id.trim().is_empty() || changes.actor.trim().is_empty() {
        return Err("Knowledge Change Set identity is invalid".into());
    }
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let committed_changes = super::action::commit(
        &mut tx,
        super::events::EventActor::new("human", Some(&changes.actor)),
        &changes.space_id,
        changes.operations,
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(committed_changes)
}
