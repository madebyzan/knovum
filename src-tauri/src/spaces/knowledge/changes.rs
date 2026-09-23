use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use super::change_commit::{self, KnowledgeChange};
use crate::spaces::db;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgeChangeSet {
    space_id: String,
    actor: String,
    operations: Vec<KnowledgeChange>,
}

#[tauri::command]
pub async fn commit_knowledge_changes<R: Runtime>(
    app: AppHandle<R>,
    changes: KnowledgeChangeSet,
) -> Result<Value, String> {
    if changes.space_id.trim().is_empty() || changes.actor.trim().is_empty() {
        return Err("Knowledge Change Set identity is invalid".into());
    }
    change_commit::validate_structure(&changes.operations)?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(&changes.space_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    if exists != 1 {
        return Err("Knowledge Change Set target Space does not exist".into());
    }
    change_commit::validate_references(&mut tx, &changes.space_id, &changes.operations).await?;
    let now = crate::clock::unix_time_millis()?;
    let result = change_commit::apply(
        &mut tx,
        &changes.space_id,
        &changes.actor,
        changes.operations,
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(result)
}
